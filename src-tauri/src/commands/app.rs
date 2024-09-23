use std::sync::Arc;

use git_version::git_version;
use log::{debug, error};
use serde::Serialize;
use tauri::{async_runtime, AppHandle, State};

use crate::{
    db::{self, revert_all_migrations},
    download::check_and_download_new_data,
    get_consumer_api_client,
    utils::switch_splashscreen_to_main,
    AppState,
};

use super::ApiError;
use crate::app_settings::AppSettings;

const GIT_VERSION: &str = git_version!();

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_downloading: bool,
    pub is_client_available: bool,
}

#[tauri::command]
pub fn get_app_version() -> String {
    String::from(GIT_VERSION)
}

#[tauri::command]
pub async fn close_welcome_screen(app_handle: AppHandle) -> Result<(), ApiError> {
    let app_settings = AppSettings::new();

    app_settings
        .safe_set(&app_handle, "termsAccepted", true)
        .map_err(|e| ApiError::Custom(format!("{}", e)))?;

    switch_splashscreen_to_main(&app_handle);

    Ok(())
}

#[tauri::command]
pub fn get_app_status(app_state: State<'_, AppState>) -> Result<StatusResponse, ApiError> {
    let downloading = app_state
        .downloading
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "downloading".into(),
        })?;

    let client_available =
        app_state
            .client_available
            .lock()
            .map_err(|_| ApiError::MutexPoisonedError {
                name: "client_available".into(),
            })?;

    Ok(StatusResponse {
        is_downloading: *downloading,
        is_client_available: *client_available,
    })
}

#[tauri::command]
pub fn clear_all_data(app_state: State<'_, AppState>) -> Result<(), ApiError> {
    {
        let mut conn = app_state
            .db
            .lock()
            .map_err(|_| ApiError::MutexPoisonedError { name: "db".into() })?;

        revert_all_migrations(&mut conn);
        db::run_migrations(&mut conn);
    }

    Ok(())
}

#[tauri::command]
pub async fn fetch_data(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), ApiError> {
    if let Some(client) = get_consumer_api_client()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        let app_state_clone = (*app_state).clone();

        async_runtime::spawn(async move {
            match check_and_download_new_data(app_handle, app_state_clone, Arc::new(client)).await {
                Ok(_) => debug!("Data download tasks completed successfully"),
                Err(e) => {
                    error!("Data download tasks panicked: {:?}", e);
                    // Handle the panic (e.g., restart the task, log the error, etc.)
                }
            }
        });
    }

    Ok(())
}
