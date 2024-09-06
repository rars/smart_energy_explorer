// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;

use data::consumption::ConsumptionRepository;
use data::consumption::RepositoryError;
use data::consumption::SqliteElectricityConsumptionRepository;
use data::consumption::SqliteGasConsumptionRepository;
use data::energy_profile::EnergyProfile;
use data::energy_profile::EnergyProfileRepository;
use data::energy_profile::SqliteEnergyProfileRepository;

use diesel::SqliteConnection;
use n3rgy::{
    Client, ElectricityConsumption, EnvironmentAuthorizationProvider, GasConsumption,
    GetRecordsError,
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

#[derive(Clone)]
struct AppState {
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    db: Arc<Mutex<SqliteConnection>>,
    downloading: Arc<Mutex<bool>>,
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
    #[error("n3rgy client error: {0}")]
    ClientError(#[from] GetRecordsError),
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

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(db_connection_clone);

        repository.get_daily(start, end)
    })
    .await;

    if let Ok(ans) = result {
        match ans {
            Ok(ans) => {
                return Ok(ans
                    .iter()
                    .map(|x| DailyElectricityConsumption {
                        timestamp: x.0,
                        value: x.1,
                    })
                    .collect());
            }
            Err(err) => {
                return Err(ApiError::Custom(format!("Database query failed: {}", err)));
            }
        }
    } else {
        return Err(ApiError::Custom("Database query failed 2".into()));
    }
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

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteElectricityConsumptionRepository::new(db_connection_clone);

        repository.get_monthly(start, end)
    })
    .await;

    if let Ok(ans) = result {
        match ans {
            Ok(ans) => {
                return Ok(ans
                    .iter()
                    .map(|x| MonthlyElectricityConsumption {
                        timestamp: x.0,
                        value: x.1,
                    })
                    .collect());
            }
            Err(err) => {
                return Err(ApiError::Custom(format!("Database query failed: {}", err)));
            }
        }
    } else {
        return Err(ApiError::Custom("Database query failed 2".into()));
    }
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

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_raw(start, end)
    })
    .await;

    if let Ok(ans) = result {
        match ans {
            Ok(ans) => {
                return Ok(ans
                    .iter()
                    .map(|x| GasConsumption {
                        timestamp: x.timestamp,
                        value: x.energy_consumption_m3,
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
async fn get_daily_gas_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyGasConsumption>, ApiError> {
    debug!("get_daily_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_daily(start, end)
    })
    .await;

    if let Ok(ans) = result {
        match ans {
            Ok(ans) => {
                return Ok(ans
                    .iter()
                    .map(|x| DailyGasConsumption {
                        timestamp: x.0,
                        value: x.1,
                    })
                    .collect());
            }
            Err(err) => {
                return Err(ApiError::Custom(format!("Database query failed: {}", err)));
            }
        }
    } else {
        return Err(ApiError::Custom("Database query failed 2".into()));
    }
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

    let result = async_runtime::spawn_blocking(move || {
        let repository = SqliteGasConsumptionRepository::new(db_connection_clone);

        repository.get_monthly(start, end)
    })
    .await;

    if let Ok(ans) = result {
        match ans {
            Ok(ans) => {
                return Ok(ans
                    .iter()
                    .map(|x| MonthlyGasConsumption {
                        timestamp: x.0,
                        value: x.1,
                    })
                    .collect());
            }
            Err(err) => {
                return Err(ApiError::Custom(format!("Database query failed: {}", err)));
            }
        }
    } else {
        return Err(ApiError::Custom("Database query failed 2".into()));
    }
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
fn update_energy_profile_settings(
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

    Ok(())
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_downloading: bool,
}

#[tauri::command]
fn get_app_status(app_state: tauri::State<'_, AppState>) -> Result<StatusResponse, ApiError> {
    let downloading = app_state
        .downloading
        .lock()
        .map_err(|e| ApiError::Custom(format!("Could not lock downloading status: {}", e)))?;

    Ok(StatusResponse {
        is_downloading: *downloading,
    })
}

trait DataLoader<T> {
    type LoadError: Error + Send + Sync + 'static;
    type InsertError: Error + Send + Sync + 'static;

    async fn load(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<T>, Self::LoadError>;
    fn insert_data(&self, data: Vec<T>) -> Result<(), Self::InsertError>;
}

struct ElectricityConsumptionDataLoader {
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl DataLoader<ElectricityConsumption> for ElectricityConsumptionDataLoader {
    type LoadError = GetRecordsError;
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

struct GasConsumptionDataLoader {
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl DataLoader<GasConsumption> for GasConsumptionDataLoader {
    type LoadError = GetRecordsError;
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

async fn check_for_new_data<T, U>(
    app_handle: AppHandle,
    connection: Arc<Mutex<SqliteConnection>>,
    profile_name: &str,
    data_loader: T,
) -> Result<(), AppError>
where
    T: DataLoader<U>,
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

    let last_date_retrieved =
        download_history(app_handle.clone(), data_loader, until_date_time).await?;

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

async fn background_task(app_handle: AppHandle, app_state: AppState) -> Result<(), AppError> {
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

    check_for_new_data(
        app_handle.clone(),
        app_state.db.clone(),
        "electricity",
        ElectricityConsumptionDataLoader {
            client: app_state.client.clone(),
            connection: app_state.db.clone(),
        },
    )
    .await?;

    check_for_new_data(
        app_handle.clone(),
        app_state.db.clone(),
        "gas",
        GasConsumptionDataLoader {
            client: app_state.client.clone(),
            connection: app_state.db.clone(),
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
    // Sleep for a specified duration
    // tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await; // Run every 1 hour

    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct DownloadUpdateEvent {
    percentage: u32,
    message: String,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            // Define the path for the database file
            let db_path = app_data_dir.join("db.sqlite");

            let mut connection =
                db::establish_connection(db_path.to_str().expect("db path needed"));
            db::run_migrations(&mut connection);

            debug!("App data directory: {}", app_data_dir.display());

            let app_state = AppState {
                client: Arc::new(Client::new(EnvironmentAuthorizationProvider, None)),
                db: Arc::new(Mutex::new(connection)),
                downloading: Arc::new(Mutex::new(false)),
            };

            let app_state_clone = app_state.clone();

            app.manage(app_state);

            let app_handle = app.handle().clone();

            // Spawn a background thread
            async_runtime::spawn(async move {
                match background_task(app_handle, app_state_clone).await {
                    Ok(_) => debug!("Background task completed successfully"),
                    Err(e) => {
                        error!("Background task panicked: {:?}", e);
                        // Handle the panic (e.g., restart the task, log the error, etc.)
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
            get_raw_electricity_consumption,
            get_daily_electricity_consumption,
            get_monthly_electricity_consumption,
            get_raw_gas_consumption,
            get_daily_gas_consumption,
            get_monthly_gas_consumption,
            get_energy_profiles,
            update_energy_profile_settings,
            get_app_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
