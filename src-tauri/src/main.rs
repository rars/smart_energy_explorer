// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use chrono::Days;
use chrono::Local;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use diesel::SqliteConnection;
use keyring::Entry;
use log::{debug, error, info};
use n3rgy_consumer_api_client::{ConsumerApiClient, N3rgyClientError, StaticAuthorizationProvider};
use serde::Serialize;
use std::env;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{async_runtime, AppHandle, Manager};
use tauri_plugin_log::Target;
use tauri_plugin_log::TargetKind;

use commands::app::*;
use commands::electricity::*;
use commands::gas::*;
use commands::n3rgy::*;
use commands::profiles::*;
use download::check_and_download_new_data;

mod commands;
mod data;
mod db;
mod download;
mod schema;
mod utils;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<SqliteConnection>>,
    downloading: Arc<Mutex<bool>>,
    client_available: Arc<Mutex<bool>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Failed request to n3rgy API: {0}")]
    N3rgyClientError(#[from] N3rgyClientError),
    #[error("Error: {0}")]
    CustomError(String),
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DailyCost {
    date: NaiveDate,
    cost_pence: f64,
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

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusUpdateEvent {
    pub is_downloading: bool,
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
            clear_all_data,
            close_welcome_screen,
            fetch_data,
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
