use std::sync::Arc;

use git_version::git_version;
use log::{debug, error};
use serde::Serialize;
use tauri::{async_runtime, AppHandle, State};

use crate::{
    db::{self, revert_all_migrations},
    download::check_and_download_new_data,
    utils::{
        delete_credential, get_glowmarkt_data_provider, reset_mqtt_settings, save_mcp_token,
        switch_main_to_splashscreen, switch_splashscreen_to_main,
    },
    AppState, MqttMessage,
};

use super::ApiError;

const GIT_VERSION: &str = git_version!();

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub is_downloading: bool,
    pub is_client_available: bool,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigResponse {
    pub url: String,
    pub token: String,
    pub config: String,
}

#[tauri::command]
pub fn get_mcp_config(app_state: State<'_, AppState>) -> Result<McpConfigResponse, ApiError> {
    let server = app_state
        .mcp_server
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "mcp_server".into(),
        })?;
    let server = server
        .as_ref()
        .ok_or_else(|| ApiError::Custom("MCP server is unavailable".into()))?;
    Ok(mcp_config_response(&server.info()))
}

#[tauri::command]
pub fn regenerate_mcp_token(app_state: State<'_, AppState>) -> Result<McpConfigResponse, ApiError> {
    let server = app_state
        .mcp_server
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "mcp_server".into(),
        })?;
    let server = server
        .as_ref()
        .ok_or_else(|| ApiError::Custom("MCP server is unavailable".into()))?;
    let token = uuid::Uuid::new_v4().to_string();
    save_mcp_token(&token)?;
    server.replace_token(token);

    Ok(mcp_config_response(&server.info()))
}

#[tauri::command]
pub async fn enable_mcp(app_state: State<'_, AppState>) -> Result<McpConfigResponse, ApiError> {
    {
        let server = app_state
            .mcp_server
            .lock()
            .map_err(|_| ApiError::MutexPoisonedError {
                name: "mcp_server".into(),
            })?;
        if let Some(server) = server.as_ref() {
            return Ok(mcp_config_response(&server.info()));
        }
    }

    let token = crate::utils::get_or_create_mcp_token()?;
    let server = crate::mcp::start(app_state.db_pool.clone(), token)
        .await
        .map_err(ApiError::Custom)?;
    let response = mcp_config_response(&server.info());

    *app_state
        .mcp_server
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "mcp_server".into(),
        })? = Some(server);
    app_state
        .app_settings
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "app_settings".into(),
        })?
        .safe_set("mcpEnabled", true)?;

    Ok(response)
}

#[tauri::command]
pub fn disable_mcp(app_state: State<'_, AppState>) -> Result<(), ApiError> {
    let server = app_state
        .mcp_server
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "mcp_server".into(),
        })?
        .take();
    if let Some(server) = server {
        server.stop();
    }

    app_state
        .app_settings
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "app_settings".into(),
        })?
        .safe_set("mcpEnabled", false)?;

    Ok(())
}

fn mcp_config_response(server: &crate::mcp::McpServerInfo) -> McpConfigResponse {
    let config = serde_json::json!({
        "mcpServers": {
            "smart-energy-explorer": {
                "command": "npx",
                "args": [
                    "-y",
                    "mcp-remote",
                    server.url,
                    "--allow-http",
                    "--header",
                    "Authorization:${SMART_ENERGY_EXPLORER_AUTH}"
                ],
                "env": {
                    "SMART_ENERGY_EXPLORER_AUTH": format!("Bearer {}", server.token)
                }
            }
        }
    });

    McpConfigResponse {
        url: server.url.clone(),
        token: server.token.clone(),
        config: serde_json::to_string_pretty(&config).expect("MCP config is serializable"),
    }
}

#[tauri::command]
pub fn get_app_version() -> String {
    String::from(GIT_VERSION)
}

#[tauri::command]
pub async fn close_welcome_screen(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), ApiError> {
    let app_settings = app_state
        .app_settings
        .lock()
        .map_err(|_| ApiError::MutexPoisonedError {
            name: "app_settings".into(),
        })?;

    app_settings
        .safe_set("termsAccepted", true)
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
    reset_database(app_state.inner())?;

    Ok(())
}

#[tauri::command]
pub async fn reset(app_handle: AppHandle, app_state: State<'_, AppState>) -> Result<(), ApiError> {
    reset_database(app_state.inner())?;

    {
        let app_settings =
            app_state
                .app_settings
                .lock()
                .map_err(|_| ApiError::MutexPoisonedError {
                    name: "app_settings".into(),
                })?;

        app_settings
            .safe_set("termsAccepted", false)
            .map_err(|e| ApiError::Custom(format!("{}", e)))?;
    }

    tokio::task::spawn_blocking(|| {
        let credentials = [
            "glowmarkt_credentials",
            "glowmarkt_username",
            "glowmarkt_password",
            "mcp_token",
        ];

        for c in credentials {
            delete_credential(c)?;
        }

        Ok::<(), ApiError>(())
    })
    .await??;

    reset_mqtt_settings(&app_handle).await?;

    app_state
        .mqtt_message_sender
        .send(MqttMessage::SettingsUpdated)
        .await
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    switch_main_to_splashscreen(&app_handle);

    Ok(())
}

fn reset_database(app_state: &AppState) -> Result<(), ApiError> {
    let mut conn = app_state.db_pool.get()?;

    revert_all_migrations(&mut conn);
    db::run_migrations(&mut conn);

    Ok(())
}

#[tauri::command]
pub async fn fetch_data(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), ApiError> {
    let app_state_clone = (*app_state).clone();

    if let Some(data_provider) = get_glowmarkt_data_provider()
        .await
        .map_err(|e| ApiError::Custom(e.to_string()))?
    {
        async_runtime::spawn(async move {
            let arc_data_provider = Arc::new(data_provider);

            match check_and_download_new_data(app_handle, app_state_clone, arc_data_provider).await
            {
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
