use tauri::{AppHandle, State};

use crate::{
    commands::ApiError,
    utils::{get_mqtt_settings_opt, save_mqtt_credentials, MqttCredentials, MqttSettings},
    AppState, MqttMessage,
};

#[tauri::command]
pub fn get_mqtt_settings(app_state: State<'_, AppState>) -> Result<MqttSettings, ApiError> {
    if let Some(settings) = get_mqtt_settings_opt(&app_state.app_settings.lock().unwrap())
        .map_err(|e| ApiError::Custom(e.to_string()))?
    {
        return Ok(settings);
    }

    Ok(MqttSettings {
        hostname: "".to_string(),
        topic: "".to_string(),
        username: "".to_string(),
        password: "".to_string(),
    })
}

#[tauri::command]
pub async fn store_mqtt_settings(
    app_state: State<'_, AppState>,
    hostname: String,
    topic: String,
    username: String,
    password: String,
) -> Result<(), ApiError> {
    let credentials = MqttCredentials {
        username: username.trim().into(),
        password: password.trim().into(),
    };

    save_mqtt_credentials(&credentials)?;

    {
        let app_settings =
            app_state
                .app_settings
                .lock()
                .map_err(|_| ApiError::MutexPoisonedError {
                    name: "app_settings".into(),
                })?;

        app_settings
            .safe_set("mqttHostname", hostname.trim().to_string())
            .map_err(|e| ApiError::Custom(format!("{}", e)))?;

        app_settings
            .safe_set("mqttTopic", topic.trim().to_string())
            .map_err(|e| ApiError::Custom(format!("{}", e)))?;

        let updated_settings =
            get_mqtt_settings_opt(&app_settings).map_err(|e| ApiError::Custom(e.to_string()))?;

        if let Ok(mut guard) = app_state.mqtt_settings.lock() {
            *guard = updated_settings;
        }
    }

    app_state
        .mqtt_message_sender
        .send(MqttMessage::SettingsUpdated)
        .await
        .map_err(|e| ApiError::Custom(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn reset_mqtt_settings(app_handle: AppHandle) -> Result<(), ApiError> {
    crate::utils::reset_mqtt_settings(&app_handle).await?;

    Ok(())
}
