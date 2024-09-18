// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::cmp;
use std::collections::BTreeMap;
use std::error::Error;
use std::future::Future;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::Datelike;
use chrono::Days;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;

use data::consumption::ConsumptionRepository;
use data::tariff::TariffRepository;
use git_version::git_version;

use data::consumption::SqliteElectricityConsumptionRepository;
use data::consumption::SqliteGasConsumptionRepository;
use data::energy_profile::EnergyProfile;
use data::energy_profile::EnergyProfileRepository;
use data::energy_profile::SqliteEnergyProfileRepository;
use data::tariff::SqliteElectricityTariffRepository;
use data::tariff::SqliteGasTariffRepository;
use data::RepositoryError;

// use diesel::serialize::Result;
use diesel::SqliteConnection;
use keyring::Entry;
use n3rgy_consumer_api_client::{
    AuthorizationProvider, ConsumerApiClient, ElectricityConsumption, ElectricityTariff,
    GasConsumption, GasTariff, N3rgyClientError, StaticAuthorizationProvider,
};
use serde::Deserialize;
use serde::Serialize;
use tauri::async_runtime;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;

use log::{debug, error, info};
use tauri_plugin_log::Target;
use tauri_plugin_log::TargetKind;

use std::env;

mod data;
mod db;
mod schema;

const GIT_VERSION: &str = git_version!();

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<SqliteConnection>>,
    downloading: Arc<Mutex<bool>>,
    client_available: Arc<Mutex<bool>>,
}

fn parse_iso_string_to_naive_date(iso_date_str: &str) -> Result<NaiveDate, ApiError> {
    NaiveDate::parse_from_str(&iso_date_str[..10], "%Y-%m-%d").map_err(ApiError::ChronoParseError)
}

#[tauri::command]
async fn get_raw_electricity_consumption(
    app_state: tauri::State<'_, AppState>,
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

#[derive(Serialize, Debug)]
pub struct DailyElectricityConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct DailyGasConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct MonthlyElectricityConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[derive(Serialize, Debug)]
pub struct MonthlyGasConsumption {
    pub timestamp: NaiveDate,
    pub value: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Error: {0}")]
    Custom(String),
    #[error("Database error: {0}")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("Date parse error: {0}")]
    ChronoParseError(#[from] chrono::ParseError),
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Failed request to n3rgy API: {0}")]
    N3rgyClientError(#[from] N3rgyClientError),
    #[error("Mutex '{name}' is poisoned")]
    MutexPoisonedError { name: String },
}

impl serde::Serialize for ApiError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Failed request to n3rgy API: {0}")]
    N3rgyClientError(#[from] N3rgyClientError),
    #[error("Error: {0}")]
    CustomError(String),
}

#[tauri::command]
async fn get_daily_electricity_consumption(
    app_state: tauri::State<'_, AppState>,
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
async fn get_monthly_electricity_consumption(
    app_state: tauri::State<'_, AppState>,
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
async fn get_raw_gas_consumption(
    app_state: tauri::State<'_, AppState>,
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
async fn get_daily_gas_consumption(
    app_state: tauri::State<'_, AppState>,
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
async fn get_monthly_gas_consumption(
    app_state: tauri::State<'_, AppState>,
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct StandingCharge {
    start_date: NaiveDateTime,
    standing_charge_pence: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UnitPrice {
    price_effective_time: NaiveDateTime,
    unit_price_pence: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TariffHistoryResponse {
    standing_charges: Vec<StandingCharge>,
    unit_prices: Vec<UnitPrice>,
}

#[tauri::command]
async fn get_electricity_tariff_history(
    app_state: tauri::State<'_, AppState>,
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
async fn get_gas_tariff_history(
    app_state: tauri::State<'_, AppState>,
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
fn get_energy_profiles(
    app_state: tauri::State<'_, AppState>,
) -> Result<Vec<EnergyProfile>, ApiError> {
    let repository = SqliteEnergyProfileRepository::new(app_state.db.clone());

    repository
        .get_all_energy_profiles()
        .map_err(|e| ApiError::Custom(format!("Database query failed: {}", e)))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnergyProfileUpdateParam {
    pub energy_profile_id: i32,
    pub is_active: bool,
    pub start_date: String,
}

#[tauri::command]
async fn update_energy_profile_settings(
    app_handle: AppHandle,
    app_state: tauri::State<'_, AppState>,
    energy_profile_updates: Vec<EnergyProfileUpdateParam>,
) -> Result<(), ApiError> {
    let update_settings: Result<Vec<_>, ApiError> = energy_profile_updates
        .iter()
        .map(|x| {
            Ok((
                x.energy_profile_id,
                x.is_active,
                parse_iso_string_to_naive_date(&x.start_date)?,
            ))
        })
        .collect();

    let repository = SqliteEnergyProfileRepository::new(app_state.db.clone());

    for (energy_profile_id, is_active, start) in update_settings? {
        debug!("Updating {}, {}, {}", energy_profile_id, is_active, start);

        let _ = repository.update_energy_profile_settings(
            energy_profile_id,
            is_active,
            start.into(),
        )?;
    }

    let app_state_clone = (*app_state).clone();

    if let Some(client) = get_consumer_api_client()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        spawn_download_tasks(app_handle, app_state_clone, client)
            .map_err(|e| ApiError::Custom(e.to_string()))?;
    }

    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_downloading: bool,
    pub is_client_available: bool,
}

#[tauri::command]
fn get_app_status(app_state: tauri::State<'_, AppState>) -> Result<StatusResponse, ApiError> {
    let downloading = app_state
        .downloading
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "downloading".into(),
        })?;

    let client_available =
        app_state
            .client_available
            .lock()
            .map_err(|_| ApiError::MutexPoisonedError {
                name: "client_available".into(),
            })?;

    Ok(StatusResponse {
        is_downloading: *downloading,
        is_client_available: *client_available,
    })
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DailyCost {
    date: NaiveDate,
    cost_pence: f64,
}

#[tauri::command]
async fn get_electricity_cost_history(
    app_state: tauri::State<'_, AppState>,
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

    let mut current_standing_charge = 0f64;
    let mut current_unit_price = 0f64;

    let mut daily_costs: Vec<DailyCost> = vec![];

    for c in consumption {
        if let Some(standing_charge) = standing_charges_map
            .range(..=NaiveDateTime::from(c.0))
            .next_back()
            .map(|(_, v)| v)
        {
            current_standing_charge = *standing_charge;
        } else {
            info!("Could not find a standing charge!!!");
            continue;
        }

        if let Some(unit_price) = unit_prices_map
            .range(..=NaiveDateTime::from(c.0))
            .next_back()
            .map(|(_, v)| v)
        {
            current_unit_price = *unit_price;
        } else {
            info!("Could not find a unit price!!!");
            continue;
        }

        daily_costs.push(DailyCost {
            date: c.0,
            cost_pence: current_standing_charge + (c.1 * current_unit_price),
        });
    }

    Ok(daily_costs)
}

#[tauri::command]
async fn get_gas_cost_history(
    app_state: tauri::State<'_, AppState>,
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

    let mut current_standing_charge = 0f64;
    let mut current_unit_price = 0f64;

    let mut daily_costs: Vec<DailyCost> = vec![];

    for c in consumption {
        if let Some(standing_charge) = standing_charges_map
            .range(..=NaiveDateTime::from(c.0))
            .next_back()
            .map(|(_, v)| v)
        {
            current_standing_charge = *standing_charge;
        } else {
            info!("Could not find a standing charge!!!");
            continue;
        }

        if let Some(unit_price) = unit_prices_map
            .range(..=NaiveDateTime::from(c.0))
            .next_back()
            .map(|(_, v)| v)
        {
            current_unit_price = *unit_price;
        } else {
            info!("Could not find a unit price!!!");
            continue;
        }

        debug!(
            "Daily cost: {} + {}",
            current_standing_charge, current_unit_price
        );

        daily_costs.push(DailyCost {
            date: c.0,
            cost_pence: current_standing_charge + (c.1 * current_unit_price),
        });
    }

    Ok(daily_costs)
}

#[tauri::command]
async fn store_api_key(
    app_handle: AppHandle,
    app_state: tauri::State<'_, AppState>,
    api_key: String,
) -> Result<(), ApiError> {
    let entry = Entry::new("n3rgy.rars.github.io", "api_key")
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    entry
        .set_password(&api_key)
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    if let Some(client) = get_consumer_api_client()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        spawn_download_tasks(app_handle, (*app_state).clone(), client)
            .map_err(|e| ApiError::Custom(e.to_string()))?;
    }

    Ok(())
}

#[tauri::command]
fn get_api_key() -> Result<String, ApiError> {
    if let Some(api_key) =
        get_api_key_opt().map_err(|e| ApiError::Custom(format!("Could not get API key: {}", e)))?
    {
        return Ok(api_key);
    }

    Ok("".into())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResponse {
    active: bool,
}

#[tauri::command]
async fn test_connection() -> Result<ConnectionTestResponse, ApiError> {
    if let Some(_) = get_consumer_api_client()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        return Ok(ConnectionTestResponse { active: true });
    }

    Ok(ConnectionTestResponse { active: false })
}

async fn get_consumer_api_client(
) -> Result<Option<ConsumerApiClient<StaticAuthorizationProvider>>, AppError> {
    if let Some(api_key) = get_api_key_opt()? {
        let ap = StaticAuthorizationProvider::new(api_key);
        let client = ConsumerApiClient::new(ap, None);

        let today = Local::now().date_naive();
        let tomorrow = today.checked_add_days(Days::new(1)).unwrap();

        if let Ok(_) = client.get_electricity_tariff(today, tomorrow).await {
            return Ok(Some(client));
        }

        if let Ok(_) = client.get_gas_tariff(today, tomorrow).await {
            return Ok(Some(client));
        }
    }

    Ok(None)
}

#[tauri::command]
async fn close_welcome_screen(app_handle: AppHandle) -> Result<(), ApiError> {
    let splash_window = app_handle.get_webview_window("splashscreen").unwrap();
    let main_window = app_handle.get_webview_window("main").unwrap();
    splash_window.close().unwrap();
    main_window.show().unwrap();

    Ok(())
}

fn get_api_key_opt() -> Result<Option<String>, AppError> {
    let entry = Entry::new("n3rgy.rars.github.io", "api_key")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    match entry.get_password() {
        Ok(password) => return Ok(Some(password)),
        Err(e) => {
            return match e {
                keyring::Error::NoEntry => Ok(None),
                _ => Err(AppError::CustomError(e.to_string())),
            }
        }
    }
}

fn spawn_download_tasks(
    app_handle: AppHandle,
    app_state: AppState,
    client: ConsumerApiClient<StaticAuthorizationProvider>,
) -> Result<(), AppError> {
    info!("Spawning download tasks");
    let client = Arc::new(client);

    async_runtime::spawn(async move {
        match check_and_download_new_data(app_handle, app_state, client).await {
            Ok(_) => debug!("Data download tasks completed successfully"),
            Err(e) => {
                error!("Data download tasks panicked: {:?}", e);
                // Handle the panic (e.g., restart the task, log the error, etc.)
            }
        }
    });

    Ok(())
}

trait DataLoader<T> {
    type LoadError: Error + Send + Sync + 'static;
    type InsertError: Error + Send + Sync + 'static;

    async fn load(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<T>, Self::LoadError>;
    fn insert_data(&self, data: Vec<T>) -> Result<(), Self::InsertError>;
}

#[derive(Clone)]
struct ElectricityConsumptionDataLoader<T>
where
    T: AuthorizationProvider,
{
    client: Arc<ConsumerApiClient<T>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<ElectricityConsumption> for ElectricityConsumptionDataLoader<T>
where
    T: AuthorizationProvider,
{
    type LoadError = N3rgyClientError;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityConsumption>, Self::LoadError> {
        Ok(self.client.get_electricity_consumption(start, end).await?)
    }

    fn insert_data(&self, data: Vec<ElectricityConsumption>) -> Result<(), Self::InsertError> {
        SqliteElectricityConsumptionRepository::new(self.connection.clone()).insert(data)?;

        Ok(())
    }
}

#[derive(Clone)]
struct ElectricityTariffDataLoader<T>
where
    T: AuthorizationProvider,
{
    client: Arc<ConsumerApiClient<T>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<ElectricityTariff> for ElectricityTariffDataLoader<T>
where
    T: AuthorizationProvider,
{
    type LoadError = N3rgyClientError;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityTariff>, Self::LoadError> {
        Ok(self.client.get_electricity_tariff(start, end).await?)
    }

    fn insert_data(&self, data: Vec<ElectricityTariff>) -> Result<(), Self::InsertError> {
        SqliteElectricityTariffRepository::new(self.connection.clone()).insert(data)?;

        Ok(())
    }
}

#[derive(Clone)]
struct GasConsumptionDataLoader<T>
where
    T: AuthorizationProvider,
{
    client: Arc<ConsumerApiClient<T>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<GasConsumption> for GasConsumptionDataLoader<T>
where
    T: AuthorizationProvider,
{
    type LoadError = N3rgyClientError;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasConsumption>, Self::LoadError> {
        Ok(self.client.get_gas_consumption(start, end).await?)
    }

    fn insert_data(&self, data: Vec<GasConsumption>) -> Result<(), Self::InsertError> {
        SqliteGasConsumptionRepository::new(self.connection.clone()).insert(data)?;

        Ok(())
    }
}

#[tauri::command]
fn get_app_version() -> String {
    String::from(GIT_VERSION)
}

#[derive(Clone)]
struct GasTariffDataLoader<T>
where
    T: AuthorizationProvider,
{
    client: Arc<ConsumerApiClient<T>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<GasTariff> for GasTariffDataLoader<T>
where
    T: AuthorizationProvider,
{
    type LoadError = N3rgyClientError;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasTariff>, Self::LoadError> {
        Ok(self.client.get_gas_tariff(start, end).await?)
    }

    fn insert_data(&self, data: Vec<GasTariff>) -> Result<(), Self::InsertError> {
        SqliteGasTariffRepository::new(self.connection.clone()).insert(data)?;

        Ok(())
    }
}

async fn download_history<T, U>(
    app_handle: AppHandle,
    data_loader: T,
    until_date_time: NaiveDateTime,
) -> Result<NaiveDate, AppError>
where
    T: DataLoader<U>,
    T::LoadError: Error + Send + Sync + 'static,
{
    let until_date = until_date_time.date();

    let today = Local::now().naive_local().date();
    let mut end_date = today;
    let mut start_of_period =
        NaiveDate::from_ymd_opt(end_date.year(), end_date.month(), 1).unwrap();

    if start_of_period < until_date {
        start_of_period = until_date;
    }

    let total_days = today.signed_duration_since(until_date).num_days();

    while start_of_period >= until_date && start_of_period < end_date {
        let records = data_loader
            .load(start_of_period, end_date)
            .await
            .map_err(|e| AppError::CustomError(format!("Error while loading data: {}", e)))?;

        info!(
            "For {} to {}, inserting {} records.",
            start_of_period,
            end_date,
            records.len()
        );

        if records.len() > 0 {
            data_loader
                .insert_data(records)
                .map_err(|e| AppError::CustomError(format!("Error while inserting data: {}", e)))?;
        }

        end_date = start_of_period;
        let end_of_previous_month =
            NaiveDate::from_ymd_opt(start_of_period.year(), start_of_period.month(), 1).unwrap()
                - Duration::days(1);
        start_of_period = NaiveDate::from_ymd_opt(
            end_of_previous_month.year(),
            end_of_previous_month.month(),
            1,
        )
        .unwrap();

        if start_of_period < until_date {
            start_of_period = until_date;
        }

        let days_remaining = end_date.signed_duration_since(until_date).num_days();

        let percentage = 100.0 * (1.0 - (days_remaining as f64 / total_days as f64));

        emit_event(
            &app_handle,
            "downloadUpdate",
            DownloadUpdateEvent {
                percentage: percentage.round() as u32,
                message: format!("Downloading {}...", percentage),
            },
        )?;
    }

    emit_event(
        &app_handle,
        "downloadUpdate",
        DownloadUpdateEvent {
            percentage: 100,
            message: "Download complete".into(),
        },
    )?;

    Ok(today)
}

fn get_or_create_energy_profile(
    connection: Arc<Mutex<SqliteConnection>>,
    name: &str,
) -> Result<EnergyProfile, AppError> {
    let repository = SqliteEnergyProfileRepository::new(connection);

    repository.get_energy_profile(name).or_else(|get_error| {
        repository
            .create_energy_profile(name)
            .map_err(|create_error| {
                AppError::CustomError(format!(
                    "Failed to fetch profile {}, get error: {}, create error: {}",
                    name, get_error, create_error
                ))
            })
    })
}

async fn check_for_new_data<F, Fut>(
    connection: Arc<Mutex<SqliteConnection>>,
    profile_name: &str,
    download_action: F,
) -> Result<(), AppError>
where
    F: FnOnce(NaiveDateTime) -> Fut,
    Fut: Future<Output = Result<NaiveDate, AppError>>,
{
    let profile = get_or_create_energy_profile(connection.clone(), profile_name)?;

    if !profile.is_active {
        info!(
            "{} profile is not active. Will not download historical data.",
            profile_name
        );
        return Ok(());
    }

    let until_date_time = profile.last_date_retrieved.unwrap_or(profile.start_date);

    let last_date_retrieved = download_action(until_date_time).await?;

    let repository = SqliteEnergyProfileRepository::new(connection);

    repository
        .update_energy_profile(
            profile.energy_profile_id,
            profile.is_active,
            profile.start_date,
            last_date_retrieved.into(),
        )
        .map_err(|error| {
            AppError::CustomError(format!(
                "Failed to update {} consumption profile, error: {}",
                profile_name, error
            ))
        })?;

    info!("Successfully updated {} consumption profile", profile_name);

    Ok(())
}

fn emit_event<T>(app_handle: &AppHandle, event: &str, payload: T) -> Result<(), AppError>
where
    T: Serialize + Clone,
{
    app_handle
        .emit(event, payload)
        .map_err(|e| AppError::CustomError(format!("Could not emit {} event: {}", event, e)))?;

    Ok(())
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusUpdateEvent {
    pub is_downloading: bool,
}

async fn check_and_download_new_data<T>(
    app_handle: AppHandle,
    app_state: AppState,
    client: Arc<ConsumerApiClient<T>>,
) -> Result<(), AppError>
where
    T: AuthorizationProvider,
{
    debug!("Background task running");

    {
        let mut downloading = app_state
            .downloading
            .lock()
            .map_err(|e| AppError::CustomError(format!("Failed to acquire lock, error: {}", e)))?;

        *downloading = true;
    }

    emit_event(
        &app_handle,
        "appStatusUpdate",
        AppStatusUpdateEvent {
            is_downloading: true,
        },
    )?;

    let electricity_consumption_data_loader = ElectricityConsumptionDataLoader {
        client: client.clone(),
        connection: app_state.db.clone(),
    };

    let electricity_tariff_data_loader = ElectricityTariffDataLoader {
        client: client.clone(),
        connection: app_state.db.clone(),
    };

    let app_handle_clone = app_handle.clone();

    check_for_new_data(
        app_state.db.clone(),
        "electricity",
        move |until_date_time| async move {
            let date_one = download_history(
                app_handle_clone.clone(),
                electricity_consumption_data_loader,
                until_date_time,
            )
            .await?;

            let date_two = download_history(
                app_handle_clone,
                electricity_tariff_data_loader,
                until_date_time,
            )
            .await?;

            Ok(cmp::max(date_one, date_two))
        },
    )
    .await?;

    let gas_consumption_data_loader = GasConsumptionDataLoader {
        client: client.clone(),
        connection: app_state.db.clone(),
    };

    let gas_tariff_data_loader = GasTariffDataLoader {
        client: client.clone(),
        connection: app_state.db.clone(),
    };

    let app_handle_clone = app_handle.clone();

    check_for_new_data(
        app_state.db.clone(),
        "gas",
        move |until_date_time| async move {
            let date_one = download_history(
                app_handle_clone.clone(),
                gas_consumption_data_loader,
                until_date_time,
            )
            .await?;

            let date_two =
                download_history(app_handle_clone, gas_tariff_data_loader, until_date_time).await?;

            Ok(cmp::max(date_one, date_two))
        },
    )
    .await?;

    {
        let mut downloading = app_state
            .downloading
            .lock()
            .map_err(|e| AppError::CustomError(format!("Failed to acquire lock, error: {}", e)))?;

        *downloading = false;
    }

    emit_event(
        &app_handle,
        "appStatusUpdate",
        AppStatusUpdateEvent {
            is_downloading: false,
        },
    )?;

    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct DownloadUpdateEvent {
    percentage: u32,
    message: String,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            debug!("App data directory: {}", app_data_dir.display());

            let db_path = app_data_dir.join("db.sqlite");

            let mut connection =
                db::establish_connection(db_path.to_str().expect("db path needed"));
            db::run_migrations(&mut connection);

            let app_state = AppState {
                db: Arc::new(Mutex::new(connection)),
                downloading: Arc::new(Mutex::new(false)),
                client_available: Arc::new(Mutex::new(false)),
            };

            let app_handle_clone = app.handle().clone();
            let app_state_clone = app_state.clone();

            app.manage(app_state);

            async_runtime::spawn(async move {
                if let Ok(Some(client)) = get_consumer_api_client().await {
                    let splash_window =
                        app_handle_clone.get_webview_window("splashscreen").unwrap();
                    let main_window = app_handle_clone.get_webview_window("main").unwrap();
                    splash_window.close().unwrap();
                    main_window.show().unwrap();

                    {
                        let mut client_available = app_state_clone.client_available.lock().unwrap();
                        *client_available = true;
                    }

                    if let Err(e) = spawn_download_tasks(app_handle_clone, app_state_clone, client)
                    {
                        error!("Failed to spawn download tasks: {}", e);
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            close_welcome_screen,
            get_api_key,
            get_app_status,
            get_app_version,
            get_daily_electricity_consumption,
            get_daily_gas_consumption,
            get_electricity_cost_history,
            get_electricity_tariff_history,
            get_energy_profiles,
            get_gas_cost_history,
            get_gas_tariff_history,
            get_monthly_electricity_consumption,
            get_monthly_gas_consumption,
            get_raw_electricity_consumption,
            get_raw_gas_consumption,
            store_api_key,
            test_connection,
            update_energy_profile_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
