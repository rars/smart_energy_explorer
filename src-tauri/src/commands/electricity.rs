use std::collections::BTreeMap;

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike};
use chrono_tz::Europe::London;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use tauri::{async_runtime, State};

use super::tariff::{DailyCost, StandingCharge, TariffHistoryResponse, UnitPrice};
use crate::{
    data::{
        consumption::{ConsumptionRepository, SqliteElectricityConsumptionRepository},
        tariff::{SqliteElectricityTariffRepository, TariffRepository},
    },
    utils::parse_iso_string_to_naive_date,
    AppState,
};

use super::ApiError;

#[derive(Serialize, PartialEq, Debug)]
pub struct ElectricityConsumption {
    #[serde(serialize_with = "crate::serde_utils::serialize_naive_as_utc")]
    pub timestamp: NaiveDateTime,
    pub value: i64,
}

#[derive(Serialize, Debug)]
pub struct DailyElectricityConsumption {
    pub timestamp: NaiveDate,
    pub value: i64,
}

#[derive(Serialize, Debug)]
pub struct MonthlyElectricityConsumption {
    pub timestamp: NaiveDate,
    pub value: i64,
}

/// A time-of-use unit-price window. Times are minutes after midnight in
/// Europe/London time. `day_of_week` uses 0 for Sunday through 6 for Saturday;
/// omitting it makes the window apply every day.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeOfUseRate {
    pub start_minutes: u16,
    pub end_minutes: u16,
    pub unit_price_pence: f64,
    pub day_of_week: Option<u8>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityTariffEstimateRequest {
    pub default_unit_price_pence: f64,
    pub rates: Vec<TimeOfUseRate>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ElectricityTariffEstimate {
    pub total_consumption_kwh: f64,
    pub total_cost_pence: f64,
    pub average_unit_price_pence: f64,
}

fn validate_tariff(tariff: &ElectricityTariffEstimateRequest) -> Result<(), ApiError> {
    if !tariff.default_unit_price_pence.is_finite() || tariff.default_unit_price_pence < 0.0 {
        return Err(ApiError::Custom(
            "The fallback unit price must be a non-negative number".into(),
        ));
    }

    for rate in &tariff.rates {
        if rate.start_minutes >= 24 * 60
            || rate.end_minutes >= 24 * 60
            || rate.start_minutes == rate.end_minutes
            || !rate.unit_price_pence.is_finite()
            || rate.unit_price_pence < 0.0
            || rate.day_of_week.is_some_and(|day| day > 6)
        {
            return Err(ApiError::Custom(
                "Each tariff window needs valid times, a non-negative price, and a valid day"
                    .into(),
            ));
        }
    }

    Ok(())
}

fn rate_applies(rate: &TimeOfUseRate, timestamp: NaiveDateTime) -> bool {
    let local_time = timestamp.and_utc().with_timezone(&London);
    let minute = (local_time.hour() * 60 + local_time.minute()) as u16;
    // For a window crossing midnight, the early-hours portion belongs to the
    // weekday on which that window started (e.g. a Monday 22:00–06:00 rate
    // includes Tuesday at 03:00).
    let rate_day = if rate.start_minutes > rate.end_minutes && minute < rate.end_minutes {
        local_time - Duration::days(1)
    } else {
        local_time
    };
    let day_matches = rate
        .day_of_week
        .is_none_or(|day| day == rate_day.weekday().num_days_from_sunday() as u8);
    let time_matches = if rate.start_minutes < rate.end_minutes {
        minute >= rate.start_minutes && minute < rate.end_minutes
    } else {
        // A window such as 22:00–06:00 spans midnight.
        minute >= rate.start_minutes || minute < rate.end_minutes
    };

    day_matches && time_matches
}

fn price_for_timestamp(tariff: &ElectricityTariffEstimateRequest, timestamp: NaiveDateTime) -> f64 {
    // Day-specific rates intentionally win over everyday rates. Within each
    // group, the first matching row wins, making overlaps predictable.
    tariff
        .rates
        .iter()
        .find(|rate| rate.day_of_week.is_some() && rate_applies(rate, timestamp))
        .or_else(|| {
            tariff
                .rates
                .iter()
                .find(|rate| rate_applies(rate, timestamp))
        })
        .map(|rate| rate.unit_price_pence)
        .unwrap_or(tariff.default_unit_price_pence)
}

#[tauri::command]
pub async fn estimate_electricity_tariff_cost(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
    tariff: ElectricityTariffEstimateRequest,
) -> Result<ElectricityTariffEstimate, ApiError> {
    validate_tariff(&tariff)?;

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;
    let connection_pool_clone = app_state.db_pool.clone();

    let consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(connection_pool_clone);
        repository.get_raw(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    let total_consumption_kwh = consumption
        .iter()
        .map(|record| record.energy_consumption_wh as f64 / 1000.0)
        .sum::<f64>();
    let total_cost_pence = consumption
        .iter()
        .map(|record| {
            (record.energy_consumption_wh as f64 / 1000.0)
                * price_for_timestamp(&tariff, record.timestamp)
        })
        .sum::<f64>();

    Ok(ElectricityTariffEstimate {
        total_consumption_kwh,
        total_cost_pence,
        average_unit_price_pence: if total_consumption_kwh > 0.0 {
            total_cost_pence / total_consumption_kwh
        } else {
            0.0
        },
    })
}

#[tauri::command]
pub async fn get_raw_electricity_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ElectricityConsumption>, ApiError> {
    debug!(
        "get_raw_electricity_consumption({}, {}) called",
        start_date, end_date
    );

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let connection_pool_clone = app_state.db_pool.clone();

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(connection_pool_clone);

        repository.get_raw(start, end)
    })
    .await;

    if let Ok(ans) = result {
        match ans {
            Ok(ans) => {
                return Ok(ans
                    .iter()
                    .map(|x| ElectricityConsumption {
                        timestamp: x.timestamp,
                        value: x.energy_consumption_wh,
                    })
                    .collect());
            }
            Err(_) => {
                return Err(ApiError::Custom("Database query failed".into()));
            }
        }
    } else {
        return Err(ApiError::Custom("Database query failed".into()));
    }
}

#[tauri::command]
pub async fn get_daily_electricity_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyElectricityConsumption>, ApiError> {
    debug!(
        "get_daily_electricity_consumption called({}, {})",
        start_date, end_date
    );

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let connection_pool_clone = app_state.db_pool.clone();

    let daily_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(connection_pool_clone);

        repository.get_daily(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))?
    .map_err(ApiError::from)?;

    Ok(daily_consumption
        .iter()
        .map(|x| DailyElectricityConsumption {
            timestamp: x.0,
            value: x.1,
        })
        .collect())
}

#[tauri::command]
pub async fn get_monthly_electricity_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<MonthlyElectricityConsumption>, ApiError> {
    debug!(
        "get_monthly_electricity_consumption({}, {}) called",
        start_date, end_date
    );

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let connection_pool_clone = app_state.db_pool.clone();

    let monthly_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(connection_pool_clone);

        repository.get_monthly(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))?
    .map_err(ApiError::from)?;

    Ok(monthly_consumption
        .iter()
        .map(|x| MonthlyElectricityConsumption {
            timestamp: x.0,
            value: x.1,
        })
        .collect())
}

#[tauri::command]
pub async fn get_electricity_tariff_history(
    app_state: State<'_, AppState>,
) -> Result<TariffHistoryResponse, ApiError> {
    let connection_pool_clone = app_state.db_pool.clone();

    let standing_charge_history = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityTariffRepository::new(connection_pool_clone);

        repository.get_standing_charge_history()
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Error: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    let connection_pool_clone = app_state.db_pool.clone();

    let unit_price_history = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityTariffRepository::new(connection_pool_clone);

        repository.get_unit_price_history()
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Error: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    let standing_charges = standing_charge_history
        .iter()
        .map(|x| StandingCharge {
            start_date: x.start_date,
            standing_charge_pence: x.standing_charge_pence,
        })
        .collect::<Vec<_>>();

    let unit_prices = unit_price_history
        .iter()
        .map(|x| UnitPrice {
            price_effective_time: x.price_effective_time,
            unit_price_pence: x.unit_price_pence,
        })
        .collect::<Vec<_>>();

    Ok(TariffHistoryResponse {
        standing_charges,
        unit_prices,
    })
}

#[tauri::command]
pub async fn get_electricity_cost_history(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyCost>, ApiError> {
    debug!("get_electricity_cost_history called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let connection_pool_clone = app_state.db_pool.clone();

    let mut consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(connection_pool_clone);

        repository.get_daily(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Error: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    consumption.sort_by_key(|x| x.0);

    let tariff_history = get_electricity_tariff_history(app_state.clone()).await?;

    let mut standing_charges_map: BTreeMap<NaiveDateTime, f64> = BTreeMap::new();

    for sc in tariff_history.standing_charges {
        standing_charges_map.insert(sc.start_date, sc.standing_charge_pence);
    }

    let mut unit_prices_map: BTreeMap<NaiveDateTime, f64> = BTreeMap::new();

    for up in tariff_history.unit_prices {
        unit_prices_map.insert(up.price_effective_time, up.unit_price_pence);
    }

    let mut daily_costs: Vec<DailyCost> = vec![];

    for c in consumption {
        let standing_charge = standing_charges_map
            .range(..=NaiveDateTime::from(c.0))
            .next_back()
            .map(|(_, v)| *v);

        let unit_price = unit_prices_map
            .range(..=NaiveDateTime::from(c.0))
            .next_back()
            .map(|(_, v)| *v);

        if let (Some(sc), Some(up)) = (standing_charge, unit_price) {
            daily_costs.push(DailyCost {
                date: c.0,
                cost_pence: sc + (((c.1 as f64) * up) / 1000.0),
            });
        } else {
            warn!(
                "Missing a value: standing charge = {:?}, unit price = {:?}",
                standing_charge, unit_price
            );
        }
    }

    Ok(daily_costs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn day_specific_window_overrides_everyday_window() {
        let tariff = ElectricityTariffEstimateRequest {
            default_unit_price_pence: 33.68,
            rates: vec![
                TimeOfUseRate {
                    start_minutes: 12 * 60,
                    end_minutes: 15 * 60,
                    unit_price_pence: 15.0,
                    day_of_week: None,
                },
                TimeOfUseRate {
                    start_minutes: 12 * 60,
                    end_minutes: 14 * 60,
                    unit_price_pence: 0.0,
                    day_of_week: Some(0),
                },
            ],
        };
        let sunday = London
            .with_ymd_and_hms(2026, 11, 22, 13, 0, 0)
            .single()
            .unwrap()
            .naive_utc();
        let monday = London
            .with_ymd_and_hms(2026, 11, 23, 13, 0, 0)
            .single()
            .unwrap()
            .naive_utc();

        assert_eq!(price_for_timestamp(&tariff, sunday), 0.0);
        assert_eq!(price_for_timestamp(&tariff, monday), 15.0);
    }
}
