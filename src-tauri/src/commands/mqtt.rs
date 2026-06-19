use tauri::{AppHandle, State};

use crate::{
    commands::ApiError,
    utils::{
        get_mqtt_settings_opt, save_mqtt_credentials, MqttAppSettings, MqttCredentials,
        MqttSettings,
    },
    AppState, MqttMessage,
};

#[tauri::command]
pub async fn get_mqtt_settings(app_state: State<'_, AppState>) -> Result<MqttSettings, ApiError> {
    let mqtt_app_settings = {
        let app_settings = app_state.app_settings.lock().unwrap();
        MqttAppSettings::from_app_settings(&app_settings)?
    };

    if let Some(settings) = get_mqtt_settings_opt(mqtt_app_settings).await? {
        return Ok(settings);
    }

    Ok(MqttSettings {
        hostname: "".to_string(),
        topic: "".to_string(),
        gas_topic: "".to_string(),
        username: "".to_string(),
        password: "".to_string(),
    })
}

#[tauri::command]
pub async fn store_mqtt_settings(
    app_state: State<'_, AppState>,
    hostname: String,
    topic: String,
    gas_topic: String,
    username: String,
    password: String,
) -> Result<(), ApiError> {
    let credentials = MqttCredentials {
        username: username.trim().into(),
        password: password.trim().into(),
    };

    tokio::task::spawn_blocking(move || save_mqtt_credentials(&credentials)).await??;

    let mqtt_app_settings = {
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

        app_settings
            .safe_set("mqttGasTopic", gas_topic.trim().to_string())
            .map_err(|e| ApiError::Custom(format!("{}", e)))?;

        MqttAppSettings::from_app_settings(&app_settings)?
    };

    let updated_settings = get_mqtt_settings_opt(mqtt_app_settings).await?;

    if let Ok(mut guard) = app_state.mqtt_settings.lock() {
        *guard = updated_settings;
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
