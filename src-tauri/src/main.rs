// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::sync::Mutex;

use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use data::energy_profile::EnergyProfile;
use data::energy_profile::EnergyProfileRepository;
use data::energy_profile::SqliteEnergyProfileRepository;
use db::create_energy_profile;
use db::get_energy_profile;
use db::insert_electricity_consumption;
use db::insert_gas_consumption;
use db::update_energy_profile;
use diesel::SqliteConnection;
use n3rgy::{
    Client, ElectricityConsumption, EnvironmentAuthorizationProvider, GasConsumption,
    GetRecordsError,
};
use serde::Serialize;
use tauri::async_runtime;
use tauri::AppHandle;
use tauri::Emitter;
use tauri::Manager;

use log::{debug, error, info, warn};
use tauri_plugin_log::Target;
use tauri_plugin_log::TargetKind;

use std::env;

mod data;
mod db;
mod schema;

struct AppState {
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    db: Arc<Mutex<SqliteConnection>>,
}

fn parse_iso_string_to_naive_date(iso_date_str: &str) -> Result<NaiveDate, GetRecordsError> {
    NaiveDate::parse_from_str(&iso_date_str[..10], "%Y-%m-%d").map_err(GetRecordsError::ChronoParse)
}

#[tauri::command]
async fn get_raw_electricity_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<ElectricityConsumption>, GetRecordsError> {
    debug!("get_raw_electricity_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let mut db_connection = db_connection_clone.lock().unwrap();

        db::get_raw_electricity_consumption(&mut *db_connection, start, end)
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
                return Err(GetRecordsError::Custom("Database query failed".into()));
            }
        }
    } else {
        return Err(GetRecordsError::Custom("Database query failed".into()));
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

#[tauri::command]
async fn get_daily_electricity_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyElectricityConsumption>, GetRecordsError> {
    debug!("get_daily_electricity_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let mut db_connection = db_connection_clone.lock().unwrap();

        db::get_daily_electricity_consumption(&mut *db_connection, start, end)
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
                return Err(GetRecordsError::Custom(format!(
                    "Database query failed: {}",
                    err
                )));
            }
        }
    } else {
        return Err(GetRecordsError::Custom("Database query failed 2".into()));
    }
}

#[tauri::command]
async fn get_monthly_electricity_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<MonthlyElectricityConsumption>, GetRecordsError> {
    debug!("get_monthly_electricity_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let mut db_connection = db_connection_clone.lock().unwrap();

        db::get_monthly_electricity_consumption(&mut *db_connection, start, end)
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
                return Err(GetRecordsError::Custom(format!(
                    "Database query failed: {}",
                    err
                )));
            }
        }
    } else {
        return Err(GetRecordsError::Custom("Database query failed 2".into()));
    }
}

#[tauri::command]
async fn get_raw_gas_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<GasConsumption>, GetRecordsError> {
    debug!("get_raw_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let mut db_connection = db_connection_clone.lock().unwrap();

        db::get_raw_gas_consumption(&mut *db_connection, start, end)
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
                return Err(GetRecordsError::Custom("Database query failed".into()));
            }
        }
    } else {
        return Err(GetRecordsError::Custom("Database query failed".into()));
    }
}

#[tauri::command]
async fn get_daily_gas_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<DailyGasConsumption>, GetRecordsError> {
    debug!("get_daily_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let mut db_connection = db_connection_clone.lock().unwrap();

        db::get_daily_gas_consumption(&mut *db_connection, start, end)
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
                return Err(GetRecordsError::Custom(format!(
                    "Database query failed: {}",
                    err
                )));
            }
        }
    } else {
        return Err(GetRecordsError::Custom("Database query failed 2".into()));
    }
}

#[tauri::command]
async fn get_monthly_gas_consumption(
    app_state: tauri::State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<MonthlyGasConsumption>, GetRecordsError> {
    debug!("get_monthly_gas_consumption called");

    let start = parse_iso_string_to_naive_date(&start_date)?;
    let end = parse_iso_string_to_naive_date(&end_date)?;

    let db_connection_clone = app_state.db.clone();

    let result = async_runtime::spawn_blocking(move || {
        let mut db_connection = db_connection_clone.lock().unwrap();

        db::get_monthly_gas_consumption(&mut *db_connection, start, end)
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
                return Err(GetRecordsError::Custom(format!(
                    "Database query failed: {}",
                    err
                )));
            }
        }
    } else {
        return Err(GetRecordsError::Custom("Database query failed 2".into()));
    }
}

#[tauri::command]
fn get_energy_profiles(
    app_state: tauri::State<'_, AppState>,
) -> Result<Vec<EnergyProfile>, GetRecordsError> {
    let mut db_connection = app_state
        .db
        .lock()
        .map_err(|e| GetRecordsError::Custom(format!("Failed to lock db: {}", e)))?;

    db::get_all_energy_profiles(&mut db_connection)
        .map_err(|e| GetRecordsError::Custom(format!("Database query failed: {}", e)))
}

#[tauri::command]
fn update_energy_profile_settings(
    app_state: tauri::State<'_, AppState>,
    energy_profile_id: i32,
    is_active: bool,
    start_date: String,
) -> Result<(), GetRecordsError> {
    let start = parse_iso_string_to_naive_date(&start_date)?;

    let mut db_connection = app_state
        .db
        .lock()
        .map_err(|e| GetRecordsError::Custom(format!("Failed to lock db: {}", e)))?;

    debug!(
        "Updating {}, {}, {}",
        energy_profile_id, is_active, start_date
    );

    if let Ok(ep) = db::update_energy_profile_settings(
        &mut db_connection,
        energy_profile_id,
        is_active,
        start.into(),
    ) {
        return Ok(());
    }

    return Err(GetRecordsError::Custom("Database query failed".into()));
}

trait DataLoader<T, E> {
    async fn load(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<T>, E>;
    fn insert_data(&self, data: Vec<T>);
}

struct ElectricityConsumptionDataLoader {
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl DataLoader<ElectricityConsumption, GetRecordsError> for ElectricityConsumptionDataLoader {
    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityConsumption>, GetRecordsError> {
        Ok(self.client.get_electricity_consumption(start, end).await?)
    }

    fn insert_data(&self, data: Vec<ElectricityConsumption>) {
        let db_clone = self.connection.clone();
        let mut conn = db_clone.lock().unwrap();

        insert_electricity_consumption(&mut *conn, data);
    }
}

struct GasConsumptionDataLoader {
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl DataLoader<GasConsumption, GetRecordsError> for GasConsumptionDataLoader {
    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasConsumption>, GetRecordsError> {
        Ok(self.client.get_gas_consumption(start, end).await?)
    }

    fn insert_data(&self, data: Vec<GasConsumption>) {
        let db_clone = self.connection.clone();
        let mut conn = db_clone.lock().unwrap();

        insert_gas_consumption(&mut *conn, data);
    }
}

async fn download_history<T, U>(
    app_handle: AppHandle,
    data_loader: T,
    until_date_time: NaiveDateTime,
) -> Result<NaiveDate, GetRecordsError>
where
    T: DataLoader<U, GetRecordsError>,
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
        let records = data_loader.load(start_of_period, end_date).await?;

        info!(
            "For {} to {}, inserting {} records.",
            start_of_period,
            end_date,
            records.len()
        );

        if records.len() > 0 {
            data_loader.insert_data(records);
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

        app_handle
            .emit(
                "downloadUpdate",
                DownloadUpdateEvent {
                    percentage: percentage.round() as u32,
                    message: format!("Downloading {}...", percentage),
                },
            )
            .map_err(|e| {
                GetRecordsError::Custom(format!("Could not emit downloadUpdate event: {}", e))
            })?;
    }

    app_handle
        .emit(
            "downloadUpdate",
            DownloadUpdateEvent {
                percentage: 100,
                message: "Download complete".into(),
            },
        )
        .map_err(|e| {
            GetRecordsError::Custom(format!("Could not emit downloadUpdate event: {}", e))
        })?;

    Ok(today)
}

fn get_or_create_energy_profile(
    connection: Arc<Mutex<SqliteConnection>>,
    name: &str,
) -> Result<EnergyProfile, GetRecordsError> {
    let mut conn = connection
        .lock()
        .map_err(|e| GetRecordsError::Custom(format!("Failed to acquire db lock: {}", e)))?;

    let mut repository = SqliteEnergyProfileRepository::new(&mut conn);

    match repository.get_energy_profile(name) {
        Ok(profile) => Ok(profile),
        Err(get_error) => match repository.create_energy_profile(name) {
            Ok(profile) => Ok(profile),
            Err(create_error) => Err(GetRecordsError::Custom(format!(
                "Failed to fetch profile {}, get error: {}, create error: {}",
                name, get_error, create_error
            ))),
        },
    }
}

async fn check_for_new_data<T, U>(
    app_handle: AppHandle,
    connection: Arc<Mutex<SqliteConnection>>,
    profile_name: &str,
    data_loader: T,
) -> Result<(), GetRecordsError>
where
    T: DataLoader<U, GetRecordsError>,
{
    let profile = get_or_create_energy_profile(connection.clone(), profile_name)?;

    if profile.is_active {
        let until_date_time = profile.last_date_retrieved.unwrap_or(profile.start_date);

        let last_date_retrieved =
            download_history(app_handle.clone(), data_loader, until_date_time).await?;

        let mut conn = connection
            .lock()
            .map_err(|e| GetRecordsError::Custom(format!("Failed to lock connection: {}", e)))?;

        match update_energy_profile(
            &mut conn,
            profile.energy_profile_id,
            profile.is_active,
            profile.start_date,
            last_date_retrieved.into(),
        ) {
            Ok(_) => info!("Successfully updated {} consumption profile", profile_name),
            Err(error) => {
                return Err(GetRecordsError::Custom(format!(
                    "Failed to update {} consumption profile, error: {}",
                    profile_name, error
                )));
            }
        }
    } else {
        info!(
            "{} profile is not active. Will not download historical data.",
            profile_name
        );
    }

    Ok(())
}

async fn background_task(
    app_handle: AppHandle,
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    connection: Arc<Mutex<SqliteConnection>>,
) -> Result<(), GetRecordsError> {
    debug!("Background task running");

    check_for_new_data(
        app_handle.clone(),
        connection.clone(),
        "electricity",
        ElectricityConsumptionDataLoader {
            client: client.clone(),
            connection: connection.clone(),
        },
    )
    .await?;

    check_for_new_data(
        app_handle.clone(),
        connection.clone(),
        "gas",
        GasConsumptionDataLoader {
            client: client.clone(),
            connection: connection.clone(),
        },
    )
    .await?;

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
            };

            let client = app_state.client.clone();
            let db = app_state.db.clone();

            app.manage(app_state);

            let app_handle = app.handle().clone();

            // Spawn a background thread
            async_runtime::spawn(async move {
                match background_task(app_handle, client, db).await {
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
            update_energy_profile_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
