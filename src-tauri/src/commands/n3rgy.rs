use keyring::Entry;
use tauri::{AppHandle, State};

use crate::{
    download::spawn_download_tasks, get_consumer_api_client, utils::get_api_key_opt, AppState,
};

use super::ApiError;

#[tauri::command]
pub async fn store_api_key(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    api_key: String,
) -> Result<(), ApiError> {
    let entry = Entry::new("n3rgy.rars.github.io", "api_key")
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    entry
        .set_password(&api_key)
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    if let Some(client) = get_consumer_api_client()
        .await
        .map_err(|e| ApiError::Custom(format!("{}", e)))?
    {
        spawn_download_tasks(app_handle, (*app_state).clone(), client)
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
