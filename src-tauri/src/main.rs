// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use std::sync::Mutex;

use chrono::Datelike;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveDate;
use db::insert_electricity_consumption;
use db::insert_gas_consumption;
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

use std::env;

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
    println!("get_raw_electricity_consumption called");

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
    println!("get_daily_electricity_consumption called");

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
    println!("get_monthly_electricity_consumption called");

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
    println!("get_raw_gas_consumption called");

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
    println!("get_daily_gas_consumption called");

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
    println!("get_monthly_gas_consumption called");

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
    num_months: u32,
    data_loader: T,
) -> Result<(), GetRecordsError>
where
    T: DataLoader<U, GetRecordsError>,
{
    let today = Local::now().naive_local().date();
    let mut end_date = today;
    let mut start_of_month = NaiveDate::from_ymd_opt(end_date.year(), end_date.month(), 1).unwrap();

    for m in 0..num_months {
        let records = data_loader.load(start_of_month, end_date).await?;

        println!(
            "For {} to {}, inserting {} records.",
            start_of_month,
            end_date,
            records.len()
        );

        if records.len() > 0 {
            data_loader.insert_data(records);

            end_date = start_of_month;
            let end_of_previous_month = start_of_month - Duration::days(1);
            start_of_month = NaiveDate::from_ymd_opt(
                end_of_previous_month.year(),
                end_of_previous_month.month(),
                1,
            )
            .unwrap();
        }

        let percentage = (100 * (m + 1)) / 12;

        app_handle
            .emit(
                "downloadUpdate",
                DownloadUpdateEvent {
                    percentage: percentage,
                    message: format!("New message {}", percentage),
                },
            )
            .unwrap();
    }

    Ok(())
}

async fn background_task(
    app_handle: AppHandle,
    client: Arc<Client<EnvironmentAuthorizationProvider>>,
    connection: Arc<Mutex<SqliteConnection>>,
) -> Result<(), GetRecordsError> {
    println!("Background task running");

    let num_months = 12;

    println!("Downloading electricity consumption");

    download_history(
        app_handle.clone(),
        num_months,
        ElectricityConsumptionDataLoader {
            client: client.clone(),
            connection: connection.clone(),
        },
    )
    .await?;

    println!("Downloading gas consumption");

    download_history(
        app_handle.clone(),
        num_months,
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

            println!("App data directory: {}", app_data_dir.display());

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
                    Ok(_) => println!("Background task completed successfully"),
                    Err(e) => {
                        println!("Background task panicked: {:?}", e);
                        // Handle the panic (e.g., restart the task, log the error, etc.)
                    }
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_sql::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            get_raw_electricity_consumption,
            get_daily_electricity_consumption,
            get_monthly_electricity_consumption,
            get_raw_gas_consumption,
            get_daily_gas_consumption,
            get_monthly_gas_consumption
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
