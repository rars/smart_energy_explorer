use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value};
use tauri::{AppHandle, Manager, Wry};
use tauri_plugin_store::{with_store, StoreCollection};

use crate::AppError;

const SETTINGS_FILE: &str = "app_settings.bin";

pub struct AppSettings {
    path: PathBuf,
}

impl AppSettings {
    pub fn new() -> Self {
        let path = PathBuf::from(SETTINGS_FILE);

        Self { path }
    }

    pub fn get<T: DeserializeOwned>(
        &self,
        app_handle: &AppHandle,
        key: &str,
    ) -> Result<Option<T>, AppError> {
        let stores = app_handle.state::<StoreCollection<Wry>>();

        let result = with_store(app_handle.clone(), stores, &self.path, |store| {
            store
                .get(key)
                .cloned()
                .map(|v| from_value(v).map_err(|e| e.into()))
                .transpose()
        })
        .map_err(|e| {
            AppError::CustomError(format!("Failed to retrieve setting '{}': {}", key, e))
        })?;

        Ok(result)
    }

    pub fn safe_set<T: Serialize>(
        &self,
        app_handle: &AppHandle,
        key: &str,
        value: T,
    ) -> Result<(), AppError> {
        let stores = app_handle.state::<StoreCollection<Wry>>();

        with_store(app_handle.clone(), stores, &self.path, |store| {
            store.insert(key.into(), to_value(value)?)?;

            store.save()?;

            Ok(())
        })
        .map_err(|e| {
            AppError::CustomError(format!("Failed to store app setting '{}': {}", key, e))
        })?;

        Ok(())
    }
}
