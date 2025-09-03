use std::sync::{Arc, Mutex};

use chrono::NaiveDate;
use diesel::SqliteConnection;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    app_settings::AppSettings,
    clients::glowmarkt::GlowmarktDataProvider,
    commands::{ApiError, APP_SERVICE_NAME},
    data::energy_profile::{EnergyProfile, EnergyProfileRepository, SqliteEnergyProfileRepository},
    AppError, AppState, MqttMessage,
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

#[derive(Serialize, Deserialize)]
pub struct GlowmarktCredentials {
    pub username: String,
    pub password: String,
}

pub fn save_glowmarkt_credentials(credentials: &GlowmarktCredentials) -> Result<(), AppError> {
    let credentials_json = serde_json::to_string(credentials).map_err(|e| {
        AppError::CustomError(format!("Failed to serialize glowmarkt credentials: {}", e))
    })?;

    let credentials_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_credentials")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    credentials_entry
        .set_password(&credentials_json)
        .map_err(|e| {
            AppError::CustomError(format!(
                "Failed to save glowmarkt credentials to keychain: {}",
                e
            ))
        })?;

    Ok(())
}

pub fn get_glowmarkt_credentials_opt() -> Result<Option<GlowmarktCredentials>, AppError> {
    // Attempt to get credentials from new single entry point
    let credentials_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_credentials")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    if let Some(credentials_json) = get_entry_password(&credentials_entry)? {
        let credentials: GlowmarktCredentials =
            serde_json::from_str(&credentials_json).map_err(|e| {
                AppError::CustomError(format!(
                    "Failed to deserialize glowmarkt credentials: {}",
                    e
                ))
            })?;

        return Ok(Some(credentials));
    } else {
        // Fallback to older separate entries
        let username_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_username")
            .map_err(|e| AppError::CustomError(e.to_string()))?;

        let password_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_password")
            .map_err(|e| AppError::CustomError(e.to_string()))?;

        let username = get_entry_password(&username_entry)?;
        let password = get_entry_password(&password_entry)?;

        match (username, password) {
            (Some(username), Some(password)) => {
                let credentials = GlowmarktCredentials { username, password };

                save_glowmarkt_credentials(&credentials)?;

                username_entry.delete_credential().map_err(|e| {
                    AppError::CustomError(format!(
                        "Failed to delete glowmarkt username credential in keychain: {}",
                        e
                    ))
                })?;

                password_entry.delete_credential().map_err(|e| {
                    AppError::CustomError(format!(
                        "Failed to delete glowmarkt password credential in keychain: {}",
                        e
                    ))
                })?;

                Ok(Some(credentials))
            }
            _ => Ok(None),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct MqttCredentials {
    pub username: String,
    pub password: String,
}

pub fn get_mqtt_credentials_opt() -> Result<Option<MqttCredentials>, AppError> {
    // Attempt to get credentials from new single entry point
    let credentials_entry = Entry::new(APP_SERVICE_NAME, "mqtt_credentials")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    if let Some(credentials_json) = get_entry_password(&credentials_entry)? {
        let credentials: MqttCredentials =
            serde_json::from_str(&credentials_json).map_err(|e| {
                AppError::CustomError(format!("Failed to deserialize mqtt credentials: {}", e))
            })?;

        return Ok(Some(credentials));
    } else {
        // Fallback to older separate entries

        let username_entry = Entry::new(APP_SERVICE_NAME, "mqtt_username")
            .map_err(|e| AppError::CustomError(e.to_string()))?;

        let password_entry = Entry::new(APP_SERVICE_NAME, "mqtt_password")
            .map_err(|e| AppError::CustomError(e.to_string()))?;

        let username = get_entry_password(&username_entry)?;
        let password = get_entry_password(&password_entry)?;

        match (username, password) {
            (Some(username), Some(password)) => {
                let credentials = MqttCredentials { username, password };

                save_mqtt_credentials(&credentials)?;

                username_entry.delete_credential().map_err(|e| {
                    AppError::CustomError(format!(
                        "Failed to delete mqtt username credential in keychain: {}",
                        e
                    ))
                })?;

                password_entry.delete_credential().map_err(|e| {
                    AppError::CustomError(format!(
                        "Failed to delete mqtt password credential in keychain: {}",
                        e
                    ))
                })?;

                Ok(Some(credentials))
            }
            _ => Ok(None),
        }
    }
}

pub fn save_mqtt_credentials(credentials: &MqttCredentials) -> Result<(), AppError> {
    let credentials_json = serde_json::to_string(credentials).map_err(|e| {
        AppError::CustomError(format!("Failed to serialize mqtt credentials: {}", e))
    })?;

    let credentials_entry = Entry::new(APP_SERVICE_NAME, "mqtt_credentials")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    credentials_entry
        .set_password(&credentials_json)
        .map_err(|e| {
            AppError::CustomError(format!(
                "Failed to save mqtt credentials to keychain: {}",
                e
            ))
        })?;

    Ok(())
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MqttSettings {
    pub hostname: String,
    pub topic: String,
    pub username: String,
    pub password: String,
}

impl MqttSettings {
    pub fn is_complete(&self) -> bool {
        self.hostname.len() > 0
            && self.topic.len() > 0
            && self.username.len() > 0
            && self.password.len() > 0
    }
}

pub fn get_mqtt_settings_opt(app_settings: &AppSettings) -> Result<Option<MqttSettings>, AppError> {
    let hostname = app_settings
        .get::<String>("mqttHostname")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    let topic = app_settings
        .get::<String>("mqttTopic")
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    if let Some(credentials) =
        get_mqtt_credentials_opt().map_err(|e| AppError::CustomError(e.to_string()))?
    {
        return Ok(Some(MqttSettings {
            hostname: hostname.unwrap_or("".to_string()),
            topic: topic.unwrap_or("".to_string()),
            username: credentials.username,
            password: credentials.password,
        }));
    }

    if hostname.is_none() && topic.is_none() {
        return Ok(None);
    }

    Ok(Some(MqttSettings {
        hostname: hostname.unwrap_or("".to_string()),
        topic: topic.unwrap_or("".to_string()),
        username: "".to_string(),
        password: "".to_string(),
    }))
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

pub fn delete_credential(key_name: &str) -> Result<(), AppError> {
    let entry =
        Entry::new(APP_SERVICE_NAME, key_name).map_err(|e| AppError::CustomError(e.to_string()))?;

    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::CustomError(e.to_string())),
    }
}

pub async fn reset_mqtt_settings(app_handle: &AppHandle) -> Result<(), AppError> {
    let app_state = app_handle.state::<AppState>();

    {
        let app_settings =
            app_state
                .app_settings
                .lock()
                .map_err(|_| AppError::MutexPoisonedError {
                    name: "app_settings".into(),
                })?;

        app_settings
            .safe_set("mqttHostname", "")
            .map_err(|e| AppError::CustomError(e.to_string()))?;

        app_settings
            .safe_set("mqttTopic", "")
            .map_err(|e| AppError::CustomError(e.to_string()))?;
    }

    let credentials = ["mqtt_credentials", "mqtt_username", "mqtt_password"];

    for c in credentials {
        delete_credential(c)?;
    }

    {
        let mut mqtt_settings =
            app_state
                .mqtt_settings
                .lock()
                .map_err(|_| AppError::MutexPoisonedError {
                    name: "mqtt_settings".into(),
                })?;

        *mqtt_settings = None;
    }

    app_state
        .mqtt_message_sender
        .send(MqttMessage::SettingsUpdated)
        .await
        .map_err(|e| AppError::CustomError(e.to_string()))?;

    Ok(())
}
