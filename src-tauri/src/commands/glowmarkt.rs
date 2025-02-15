use keyring::Entry;
use tauri::{AppHandle, State};

use crate::{
    commands::ApiError,
    download::spawn_download_tasks,
    utils::{get_glowmarkt_credentials_opt, get_glowmarkt_data_provider},
    AppState,
};

use super::APP_SERVICE_NAME;

#[tauri::command]
pub async fn store_glowmarkt_credentials(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<(), ApiError> {
    let username_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_username")
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    username_entry
        .set_password(&username)
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    let password_entry = Entry::new(APP_SERVICE_NAME, "glowmarkt_password")
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    password_entry
        .set_password(&password)
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    if let Some(data_provider) = get_glowmarkt_data_provider()
        .await
        .map_err(|e| ApiError::Custom(e.to_string()))?
    {
        spawn_download_tasks(app_handle, (*app_state).clone(), data_provider)
            .map_err(|e| ApiError::Custom(e.to_string()))?;
    }

    Ok(())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlowmarktCredentialsResponse {
    username: String,
    password: String,
}

#[tauri::command]
pub fn get_glowmarkt_credentials() -> Result<GlowmarktCredentialsResponse, ApiError> {
    if let Some(credentials) =
        get_glowmarkt_credentials_opt().map_err(|e| ApiError::Custom(e.to_string()))?
    {
        return Ok(GlowmarktCredentialsResponse {
            username: credentials.username,
            password: credentials.password,
        });
    }

    Ok(GlowmarktCredentialsResponse {
        username: "".to_string(),
        password: "".to_string(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResponse {
    active: bool,
}

#[tauri::command]
pub async fn test_glowmarkt_connection() -> Result<ConnectionTestResponse, ApiError> {
    if let Some(_) = get_glowmarkt_data_provider()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        return Ok(ConnectionTestResponse { active: true });
    }

    Ok(ConnectionTestResponse { active: false })
}
