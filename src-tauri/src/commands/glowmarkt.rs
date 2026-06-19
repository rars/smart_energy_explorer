use tauri::{AppHandle, State};

use crate::{
    commands::ApiError,
    download::spawn_download_tasks,
    utils::{
        get_glowmarkt_credentials_opt, get_glowmarkt_data_provider, save_glowmarkt_credentials,
        GlowmarktCredentials,
    },
    AppState,
};

#[tauri::command]
pub async fn store_glowmarkt_credentials(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    username: String,
    password: String,
) -> Result<(), ApiError> {
    let credentials = GlowmarktCredentials { username, password };

    tokio::task::spawn_blocking(move || save_glowmarkt_credentials(&credentials)).await??;

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
pub async fn get_glowmarkt_credentials() -> Result<GlowmarktCredentialsResponse, ApiError> {
    let credentials_result =
        tokio::task::spawn_blocking(|| get_glowmarkt_credentials_opt()).await?;

    if let Some(credentials) = credentials_result? {
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
