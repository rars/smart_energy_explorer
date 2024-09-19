// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use diesel::SqliteConnection;
use log::{debug, error};
use n3rgy_consumer_api_client::N3rgyClientError;
use std::env;
use std::sync::{Arc, Mutex};
use tauri::{async_runtime, Manager};
use tauri_plugin_log::{Target, TargetKind};
use utils::{get_consumer_api_client, switch_splashscreen_to_main};

use commands::app::*;
use commands::electricity::*;
use commands::gas::*;
use commands::n3rgy::*;
use commands::profiles::*;

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
                    switch_splashscreen_to_main(&app_handle_clone);

                    {
                        let mut client_available = app_state_clone.client_available.lock().unwrap();
                        *client_available = true;
                    }

                    if let Err(e) =
                        download::spawn_download_tasks(app_handle_clone, app_state_clone, client)
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
