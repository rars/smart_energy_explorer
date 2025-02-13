use keyring::Entry;
use tauri::{AppHandle, State};

use crate::{
    download::spawn_download_tasks,
    get_consumer_api_client,
    utils::{get_api_key_opt, get_glowmarkt_data_provider},
    AppState,
};

use super::ApiError;

pub(crate) const APP_SERVICE_NAME: &str = "io.github.rars.smart_energy_explorer";

#[tauri::command]
pub async fn store_api_key(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    api_key: String,
) -> Result<(), ApiError> {
    let entry =
        Entry::new(APP_SERVICE_NAME, "api_key").map_err(|e| ApiError::Custom(e.to_string()))?;

    entry
        .set_password(&api_key)
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

#[tauri::command]
pub fn get_api_key() -> Result<String, ApiError> {
    if let Some(api_key) =
        get_api_key_opt().map_err(|e| ApiError::Custom(format!("Could not get API key: {}", e)))?
    {
        return Ok(api_key);
    }

    Ok("".into())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionTestResponse {
    active: bool,
}

#[tauri::command]
pub async fn test_connection() -> Result<ConnectionTestResponse, ApiError> {
    if let Some(_) = get_consumer_api_client()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        return Ok(ConnectionTestResponse { active: true });
    }

    Ok(ConnectionTestResponse { active: false })
}
