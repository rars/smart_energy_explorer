use std::sync::{Arc, Mutex};

use chrono::{Days, Local, NaiveDate};
use diesel::SqliteConnection;
use keyring::Entry;
use n3rgy_consumer_api_client::{ConsumerApiClient, StaticAuthorizationProvider};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    commands::ApiError,
    data::energy_profile::{EnergyProfile, EnergyProfileRepository, SqliteEnergyProfileRepository},
    AppError, APP_SERVICE_NAME,
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

pub async fn get_consumer_api_client(
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

pub fn get_api_key_opt() -> Result<Option<String>, AppError> {
    let entry = Entry::new(APP_SERVICE_NAME, "api_key")
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

pub fn switch_splashscreen_to_main(app_handle: &AppHandle) {
    let splash_window = app_handle.get_webview_window("splashscreen").unwrap();
    let main_window = app_handle.get_webview_window("main").unwrap();
    splash_window.close().unwrap();
    main_window.show().unwrap();
}
