// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_settings::{AppSettings, SETTINGS_FILE};
use clients::glowmarkt::GlowmarktDataProviderError;
use diesel::SqliteConnection;
use log::{debug, error};
use std::env;
use std::fs;
use std::sync::{Arc, Mutex};
use tauri::Window;
use tauri::{async_runtime, Manager};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_store::StoreExt;
use utils::{get_glowmarkt_data_provider, switch_splashscreen_to_main};

use commands::app::*;
use commands::electricity::*;
use commands::gas::*;
use commands::glowmarkt::*;
use commands::profiles::*;

mod app_settings;
mod clients;
mod commands;
mod data;
mod db;
mod download;
mod schema;
mod utils;

struct AppState {
    db: Arc<Mutex<SqliteConnection>>,
    downloading: Arc<Mutex<bool>>,
    client_available: Arc<Mutex<bool>>,
    app_settings: Arc<Mutex<AppSettings>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            downloading: self.downloading.clone(),
            client_available: self.client_available.clone(),
            app_settings: self.app_settings.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Failed interaction with Glowmarkt API: {0}")]
    GlowmarktApiError(#[from] GlowmarktDataProviderError),
    #[error("Error: {0}")]
    CustomError(String),
}

fn set_close_handlers(window: &Window) {
    window.on_window_event(|event| {
        if let tauri::WindowEvent::CloseRequested { .. } = event {
            std::process::exit(0);
        }
    });

    ()
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

            if !app_data_dir.exists() {
                fs::create_dir_all(&app_data_dir)
                    .expect("app data directory does not exist and cannot be created");
            }

            let db_path = app_data_dir.join("db.sqlite");

            let mut connection =
                db::establish_connection(db_path.to_str().expect("db path needed"));
            db::run_migrations(&mut connection);

            let store = app.store(SETTINGS_FILE)?;

            let app_settings = AppSettings::new(store);

            let app_state = AppState {
                db: Arc::new(Mutex::new(connection)),
                downloading: Arc::new(Mutex::new(false)),
                client_available: Arc::new(Mutex::new(false)),
                app_settings: Arc::new(Mutex::new(app_settings)),
            };

            let app_handle_clone = app.handle().clone();
            let app_state_clone = app_state.clone();

            app.manage(app_state);

            {
                let app_settings = app_state_clone.app_settings.lock().unwrap();

                if let Some(true) = app_settings.get::<bool>("termsAccepted")? {
                    switch_splashscreen_to_main(&app_handle_clone);
                }
            }

            async_runtime::spawn(async move {
                if let Ok(Some(data_provider)) = get_glowmarkt_data_provider().await {
                    {
                        let mut client_available = app_state_clone.client_available.lock().unwrap();
                        *client_available = true;
                    }

                    if let Err(e) = download::spawn_download_tasks(
                        app_handle_clone,
                        app_state_clone,
                        data_provider,
                    ) {
                        error!("Failed to spawn download tasks: {}", e);
                    }
                }
            });

            let window = app.get_window("main").unwrap();

            set_close_handlers(&window);

            let window = app.get_window("splashscreen").unwrap();

            set_close_handlers(&window);

            Ok(())
        })
        .plugin(tauri_plugin_sql::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
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
            get_app_status,
            get_app_version,
            get_daily_electricity_consumption,
            get_daily_gas_consumption,
            get_electricity_cost_history,
            get_electricity_tariff_history,
            get_energy_profiles,
            get_gas_cost_history,
            get_gas_tariff_history,
            get_glowmarkt_credentials,
            get_monthly_electricity_consumption,
            get_monthly_gas_consumption,
            get_raw_electricity_consumption,
            get_raw_gas_consumption,
            reset,
            store_glowmarkt_credentials,
            test_glowmarkt_connection,
            update_energy_profile_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
