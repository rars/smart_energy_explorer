use std::sync::{Arc, Mutex};

use chrono::{NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Europe::London;
use diesel::SqliteConnection;
use keyring_core::Entry;
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

pub fn london_midnight_as_utc(date: &NaiveDate) -> NaiveDateTime {
    let london_midnight_local = London
        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
        .unwrap();

    let london_midnight_utc = london_midnight_local.with_timezone(&Utc);

    london_midnight_utc.naive_utc()
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
    let credentials_result =
        tokio::task::spawn_blocking(|| get_glowmarkt_credentials_opt()).await?;

    if let Some(GlowmarktCredentials { username, password }) = credentials_result? {
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

fn get_keyring_entry_value(value: &str) -> Result<(Entry, Option<String>), AppError> {
    let entry =
        Entry::new(APP_SERVICE_NAME, value).map_err(|e| AppError::CustomError(e.to_string()))?;

    let password = get_entry_password(&entry)?;

    Ok((entry, password))
}

pub fn get_mqtt_credentials_opt() -> Result<Option<MqttCredentials>, AppError> {
    // Attempt to get credentials from new single entry point
    let (_, credentials_entry) = get_keyring_entry_value("mqtt_credentials")?;

    if let Some(credentials_json) = credentials_entry {
        let credentials: MqttCredentials =
            serde_json::from_str(&credentials_json).map_err(|e| {
                AppError::CustomError(format!("Failed to deserialize mqtt credentials: {}", e))
            })?;

        return Ok(Some(credentials));
    } else {
        // Fallback to older separate entries
        let (username_entry, username) = get_keyring_entry_value("mqtt_username")?;
        let (password_entry, password) = get_keyring_entry_value("mqtt_password")?;

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
    pub gas_topic: String,
    pub username: String,
    pub password: String,
}

impl MqttSettings {
    pub fn is_complete(&self) -> bool {
        self.hostname.len() > 0
            && (self.topic.len() > 0 || self.gas_topic.len() > 0)
            && self.username.len() > 0
            && self.password.len() > 0
    }
}

#[derive(Clone)]
pub struct MqttAppSettings {
    pub hostname: Option<String>,
    pub topic: Option<String>,
    pub gas_topic: Option<String>,
}

impl MqttAppSettings {
    pub fn from_app_settings(app_settings: &AppSettings) -> Result<Self, AppError> {
        let hostname = app_settings.get::<String>("mqttHostname")?;

        let topic = app_settings.get::<String>("mqttTopic")?;

        let gas_topic = app_settings.get::<String>("mqttGasTopic")?;

        Ok(MqttAppSettings {
            hostname,
            topic,
            gas_topic,
        })
    }
}

pub async fn get_mqtt_settings_opt(
    mqtt_app_settings: MqttAppSettings,
) -> Result<Option<MqttSettings>, AppError> {
    let credentials_result = tokio::task::spawn_blocking(|| get_mqtt_credentials_opt()).await?;

    if let Some(credentials) = credentials_result? {
        return Ok(Some(MqttSettings {
            hostname: mqtt_app_settings.hostname.unwrap_or("".to_string()),
            topic: mqtt_app_settings.topic.unwrap_or("".to_string()),
            gas_topic: mqtt_app_settings.gas_topic.unwrap_or("".to_string()),
            username: credentials.username,
            password: credentials.password,
        }));
    }

    if mqtt_app_settings.hostname.is_none() && mqtt_app_settings.topic.is_none() {
        return Ok(None);
    }

    Ok(Some(MqttSettings {
        hostname: mqtt_app_settings.hostname.unwrap_or("".to_string()),
        topic: mqtt_app_settings.topic.unwrap_or("".to_string()),
        gas_topic: mqtt_app_settings.gas_topic.unwrap_or("".to_string()),
        username: "".to_string(),
        password: "".to_string(),
    }))
}

fn get_entry_password(entry: &Entry) -> Result<Option<String>, AppError> {
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring_core::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::CustomError(e.to_string())),
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
        Err(keyring_core::Error::NoEntry) => Ok(()),
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

        app_settings.safe_set("mqttHostname", "")?;

        app_settings.safe_set("mqttTopic", "")?;

        app_settings.safe_set("mqttGasTopic", "")?;
    }

    tokio::task::spawn_blocking(|| {
        let credentials = ["mqtt_credentials", "mqtt_username", "mqtt_password"];

        for c in credentials {
            delete_credential(c)?;
        }

        Ok::<(), AppError>(())
    })
    .await??;

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use chrono::NaiveTime;

    fn complete_settings() -> MqttSettings {
        MqttSettings {
            hostname: "localhost".to_string(),
            topic: "test/topic".to_string(),
            gas_topic: "test/gas".to_string(),
            username: "user".to_string(),
            password: "password".to_string(),
        }
    }

    #[test]
    fn test_parse_iso_string_to_naive_date_valid() {
        let result = parse_iso_string_to_naive_date("2026-06-24T12:00:00Z");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            NaiveDate::from_ymd_opt(2026, 6, 24).unwrap()
        );
    }

    #[test]
    fn test_parse_iso_string_to_naive_date_invalid() {
        let result = parse_iso_string_to_naive_date("invalid-date");
        assert!(result.is_err());
    }

    #[test]
    fn test_mqtt_settings_is_complete_all_fields() {
        assert!(complete_settings().is_complete());
    }

    #[test]
    fn test_mqtt_settings_is_complete_only_electricity_topic() {
        let settings = MqttSettings {
            gas_topic: "".to_string(),
            ..complete_settings()
        };
        assert!(settings.is_complete());
    }

    #[test]
    fn test_mqtt_settings_is_complete_only_gas_topic() {
        let settings = MqttSettings {
            topic: "".to_string(),
            ..complete_settings()
        };
        assert!(settings.is_complete());
    }

    #[test]
    fn test_mqtt_settings_is_incomplete_missing_hostname() {
        let settings = MqttSettings {
            hostname: "".to_string(),
            ..complete_settings()
        };
        assert!(!settings.is_complete());
    }

    #[test]
    fn test_mqtt_settings_is_incomplete_missing_topics() {
        let settings = MqttSettings {
            topic: "".to_string(),
            gas_topic: "".to_string(),
            ..complete_settings()
        };
        assert!(!settings.is_complete());
    }

    #[test]
    fn test_mqtt_settings_is_incomplete_missing_username() {
        let settings = MqttSettings {
            username: "".to_string(),
            ..complete_settings()
        };
        assert!(!settings.is_complete());
    }

    #[test]
    fn test_mqtt_settings_is_incomplete_missing_password() {
        let settings = MqttSettings {
            password: "".to_string(),
            ..complete_settings()
        };
        assert!(!settings.is_complete());
    }

    #[test]
    fn test_london_midnight_during_gmt_winter() {
        let winter_date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();

        let result = london_midnight_as_utc(&winter_date);

        let expected = winter_date.and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_london_midnight_during_bst_summer() {
        let summer_date = NaiveDate::from_ymd_opt(2026, 6, 26).unwrap();

        let result = london_midnight_as_utc(&summer_date);

        let expected_date = NaiveDate::from_ymd_opt(2026, 6, 25).unwrap();
        let expected_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_london_midnight_on_clocks_forward_transition_day() {
        let transition_date = NaiveDate::from_ymd_opt(2026, 3, 29).unwrap();

        let result = london_midnight_as_utc(&transition_date);

        let expected = transition_date.and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_london_midnight_on_clocks_back_transition_day() {
        let transition_sunday = NaiveDate::from_ymd_opt(2026, 10, 25).unwrap();

        let result = london_midnight_as_utc(&transition_sunday);

        let expected_date = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        let expected_time = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
        let expected = expected_date.and_time(expected_time);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_london_midnight_day_after_clocks_back() {
        let post_transition_monday = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();

        let result = london_midnight_as_utc(&post_transition_monday);

        let expected = post_transition_monday.and_hms_opt(0, 0, 0).unwrap();
        assert_eq!(result, expected);
    }
}
