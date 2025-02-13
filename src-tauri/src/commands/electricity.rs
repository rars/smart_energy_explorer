use std::collections::BTreeMap;

use chrono::{NaiveDate, NaiveDateTime};
use log::{debug, warn};
use serde::Serialize;
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
    pub timestamp: NaiveDateTime,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct DailyElectricityConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct MonthlyElectricityConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[tauri::command]
pub async fn get_raw_electricity_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ElectricityConsumption>, ApiError> {
    debug!("get_raw_electricity_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(db_connection_clone);

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
                        value: x.energy_consumption_kwh,
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
    debug!("get_daily_electricity_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let daily_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(db_connection_clone);

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
    debug!("get_monthly_electricity_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let monthly_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(db_connection_clone);

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
    let db_connection_clone = app_state.db.clone();

    let standing_charge_history = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityTariffRepository::new(db_connection_clone);

        repository.get_standing_charge_history()
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Error: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    let db_connection_clone = app_state.db.clone();

    let unit_price_history = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityTariffRepository::new(db_connection_clone);

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

    let db_connection_clone = app_state.db.clone();

    let mut consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(db_connection_clone);

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
                cost_pence: sc + (c.1 * up),
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
