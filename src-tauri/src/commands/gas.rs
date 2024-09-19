use std::collections::BTreeMap;

use chrono::{NaiveDate, NaiveDateTime};
use log::{debug, warn};
use n3rgy_consumer_api_client::GasConsumption;
use serde::Serialize;
use tauri::{async_runtime, State};

use super::tariff::{DailyCost, StandingCharge, TariffHistoryResponse, UnitPrice};
use crate::{
    commands::ApiError,
    data::{
        consumption::{ConsumptionRepository, SqliteGasConsumptionRepository},
        tariff::{SqliteGasTariffRepository, TariffRepository},
    },
    utils::parse_iso_string_to_naive_date,
    AppState,
};

#[derive(Serialize, Debug)]
pub struct DailyGasConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct MonthlyGasConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[tauri::command]
pub async fn get_raw_gas_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<GasConsumption>, ApiError> {
    debug!("get_raw_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let raw_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_raw(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))?
    .map_err(ApiError::from)?;

    Ok(raw_consumption
        .iter()
        .map(|x| GasConsumption {
            timestamp: x.timestamp,
            value: x.energy_consumption_m3,
        })
        .collect())
}

#[tauri::command]
pub async fn get_daily_gas_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyGasConsumption>, ApiError> {
    debug!("get_daily_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let daily_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_daily(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))?
    .map_err(ApiError::from)?;

    Ok(daily_consumption
        .iter()
        .map(|x| DailyGasConsumption {
            timestamp: x.0,
            value: x.1,
        })
        .collect())
}

#[tauri::command]
pub async fn get_monthly_gas_consumption(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<MonthlyGasConsumption>, ApiError> {
    debug!("get_monthly_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let monthly_consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_monthly(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))?
    .map_err(ApiError::from)?;

    Ok(monthly_consumption
        .iter()
        .map(|x| MonthlyGasConsumption {
            timestamp: x.0,
            value: x.1,
        })
        .collect())
}

#[tauri::command]
pub async fn get_gas_tariff_history(
    app_state: State<'_, AppState>,
) -> Result<TariffHistoryResponse, ApiError> {
    let db_connection_clone = app_state.db.clone();

    let standing_charge_history = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasTariffRepository::new(db_connection_clone);

        repository.get_standing_charge_history()
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Error: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    let db_connection_clone = app_state.db.clone();

    let unit_price_history = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasTariffRepository::new(db_connection_clone);

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
pub async fn get_gas_cost_history(
    app_state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyCost>, ApiError> {
    debug!("get_gas_cost_history called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let mut consumption = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_daily(start, end)
    })
    .await
    .map_err(|e| ApiError::Custom(format!("Error: {}", e)))?
    .map_err(ApiError::RepositoryError)?;

    consumption.sort_by_key(|x| x.0);

    let tariff_history = get_gas_tariff_history(app_state.clone()).await?;

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
