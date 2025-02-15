use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use diesel::SqliteConnection;
use keyring::Entry;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    clients::glowmarkt::GlowmarktDataProvider,
    commands::{ApiError, APP_SERVICE_NAME},
    data::energy_profile::{EnergyProfile, EnergyProfileRepository, SqliteEnergyProfileRepository},
    AppError,
};

pub fn parse_iso_string_to_naive_date(iso_date_str: &str) -> Result<NaiveDate, ApiError> {
    NaiveDate::parse_from_str(&iso_date_str[..10], "%Y-%m-%d").map_err(ApiError::ChronoParseError)
}

pub fn emit_event<T>(app_handle: &AppHandle, event: &str, payload: T) -> Result<(), AppError>
where
    T: Serialize + Clone,
{
    app_handle
        .emit(event, payload)
        .map_err(|e| AppError::CustomError(format!("Could not emit {} event: {}", event, e)))?;

    Ok(())
}

pub fn get_or_create_energy_profile(
    connection: Arc<Mutex<SqliteConnection>>,
    name: &str,
    base_unit: &str,
) -> Result<EnergyProfile, AppError> {
    let repository = SqliteEnergyProfileRepository::new(connection);

    repository.get_energy_profile(name).or_else(|get_error| {
        repository
            .create_energy_profile(name, base_unit)
            .map_err(|create_error| {
                AppError::CustomError(format!(
                    "Failed to fetch profile {}, get error: {}, create error: {}",
                    name, get_error, create_error
                ))
            })
    })
}

pub async fn get_glowmarkt_data_provider() -> Result<Option<GlowmarktDataProvider>, AppError> {
    if let Some(GlowmarktCredentials { username, password }) = get_glowmarkt_credentials_opt()? {
        let data_provider = GlowmarktDataProvider::new(&username, &password)
            .await
            .map_err(|e| AppError::CustomError(e.to_string()))?;

        return Ok(Some(data_provider));
    }

    Ok(None)
}

pub struct GlowmarktCredentials {
    pub username: String,
    pub password: String,
}

pub fn get_glowmarkt_credentials_opt() -> Result<Option<GlowmarktCredentials>, AppError> {
    let username_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_username")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    let password_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_password")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    let username = get_entry_password(&username_entry)?;
    let password = get_entry_password(&password_entry)?;

    match (username, password) {
        (Some(username), Some(password)) => Ok(Some(GlowmarktCredentials { username, password })),
        _ => Ok(None),
    }
}

fn get_entry_password(entry: &Entry) -> Result<Option<String>, AppError> {
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(e) => match e {
            keyring::Error::NoEntry => Ok(None),
            _ => Err(AppError::CustomError(e.to_string())),
        },
    }
}

pub fn switch_splashscreen_to_main(app_handle: &AppHandle) {
    let splash_window = app_handle.get_webview_window("splashscreen").unwrap();
    let main_window = app_handle.get_webview_window("main").unwrap();
    splash_window.hide().unwrap();
    main_window.show().unwrap();
    main_window.eval("window.location.reload()").unwrap();
}

pub fn switch_main_to_splashscreen(app_handle: &AppHandle) {
    let splash_window = app_handle.get_webview_window("splashscreen").unwrap();
    let main_window = app_handle.get_webview_window("main").unwrap();
    main_window.hide().unwrap();
    splash_window.show().unwrap();
    splash_window.eval("window.location.reload()").unwrap();
}
