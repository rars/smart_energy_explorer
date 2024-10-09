use std::fmt::Display;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};
use tauri::Wry;
use tauri_plugin_store::Store;

use crate::AppError;

pub(crate) const SETTINGS_FILE: &str = "app_settings.bin";

type SettingsStore = Store<Wry>;

pub struct AppSettings {
    store: SettingsStore,
}

impl AppSettings {
    pub fn new(store: SettingsStore) -> Self {
        Self { store }
    }

    pub fn get<R: DeserializeOwned>(&self, key: &str) -> Result<Option<R>, AppError> {
        let result = self
            .store
            .get(key)
            .map(|v| from_value(v))
            .transpose()
            .map_err(|e| {
                AppError::CustomError(format!("Failed to retrieve setting '{}': {}", key, e))
            })?;

        Ok(result)
    }

    pub fn safe_set<R: Serialize>(&self, key: &str, value: R) -> Result<(), AppError>
    where
        R: Display + Copy,
    {
        let json_value = to_value(value).map_err(|e| {
            AppError::CustomError(format!(
                "Failed to convert '{}' to JSON value: {}",
                value, e
            ))
        })?;

        self.store.set(key, json_value);

        self.store
            .save()
            .map_err(|e| AppError::CustomError(format!("Could not save store: {}", e)))?;

        Ok(())
    }
}
