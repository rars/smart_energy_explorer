use std::time::Duration;

use log::{error, info};
use paho_mqtt::{self as mqtt, AsyncClient, AsyncReceiver, DisconnectOptionsBuilder, Message};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{
    utils::{emit_event, MqttSettings},
    AppState, MqttMessage,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityMeter {
    pub timestamp: String,
    pub energy: Energy,
    pub power: Power,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Energy {
    pub export: EnergyExport,
    pub import: EnergyImport,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnergyExport {
    pub cumulative: f64,
    pub units: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EnergyImport {
    pub cumulative: f64,
    pub day: f64,
    pub week: f64,
    pub month: f64,
    pub units: String,
    pub mpan: String,
    pub supplier: String,
    pub price: ImportPrice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ImportPrice {
    pub unitrate: f64,
    pub standingcharge: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Power {
    pub value: f64,
    pub units: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityMeterMessage {
    pub electricitymeter: ElectricityMeter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityUpdate {
    is_active: bool,
    message: ElectricityMeterMessage,
}

async fn create_mqtt_client(
    client_id: String,
    settings: &MqttSettings,
) -> Result<AsyncClient, paho_mqtt::Error> {
    let qos = 1;

    let client_options = mqtt::CreateOptionsBuilder::new()
        .server_uri(settings.hostname.clone())
        .client_id(client_id)
        .finalize();

    let client = mqtt::AsyncClient::new(client_options)?;

    let connection_options = mqtt::ConnectOptionsBuilder::new()
        .clean_session(true)
        .user_name(settings.username.clone())
        .password(settings.password.clone())
        .connect_timeout(Duration::from_secs(5))
        .automatic_reconnect(Duration::from_secs(1), Duration::from_secs(30))
        .finalize();

    client.connect(connection_options).await?;

    client.subscribe(settings.topic.clone(), qos).await?;

    Ok(client)
}

pub async fn start_mqtt_listener(
    app_handle: &AppHandle,
    mut mqtt_message_receiver: mpsc::Receiver<MqttMessage>,
) {
    info!("Starting MQTT listener thread.");

    let uuid_string = Uuid::new_v4().to_string();
    let client_id = format!("smart-energy-explorer-{}", uuid_string);
    info!("Generated Client ID: {}", client_id);

    let mut client_and_stream: Option<(AsyncClient, AsyncReceiver<Option<Message>>)> = None;

    loop {
        let (client, stream) = match client_and_stream.as_mut() {
            Some((c, s)) => (c, s),
            None => {
                let settings = {
                    let app_state = app_handle.state::<AppState>();
                    let settings = app_state.mqtt_settings.lock().unwrap();
                    settings.clone()
                };

                if let Some(settings) = settings {
                    if settings.is_complete() {
                        info!("MQTT settings are complete but client is not yet created. Creating MQTT client...");
                        match create_mqtt_client(client_id.clone(), &settings).await {
                            Ok(mut client) => {
                                let stream = client.get_stream(None);
                                info!("MQTT client and stream created");
                                client_and_stream = Some((client, stream));
                                continue;
                            }
                            Err(e) => {
                                error!("Failed to create client: {}", e);
                            }
                        };
                    } else {
                        info!("MQTT settings are not complete");
                    }
                } else {
                    info!("MQTT settings are not set")
                }

                // Sleep to avoid tight loop
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        tokio::select! {
            Some(app_message) = mqtt_message_receiver.recv() => {
                match app_message {
                    MqttMessage::SettingsUpdated => {
                        info!("MQTT settings updated, recreating client");
                        disconnect_client(client).await;
                        client_and_stream = None;
                    }
                }
            },
            message = stream.next() => {
                if let Some(Some(msg)) = message {
                    match serde_json::from_str::<ElectricityMeterMessage>(&msg.payload_str()) {
                        Ok(data) => {
                            info!("Deserialized data: {:?}", data);
                            // Now you can work with the structured 'data' object

                            emit_event(app_handle, "electricityUpdate", data);
                        }
                        Err(e) => {
                            error!("Failed to deserialize payload: {}", e);
                        }
                    }
                }
            },
        }
    }
}

async fn disconnect_client(client: &AsyncClient) {
    let opts = DisconnectOptionsBuilder::new()
        .timeout(Duration::from_secs(5))
        .finalize();

    match client.disconnect(Some(opts)).await {
        Ok(_) => info!("Successfully disconnected MQTT client."),
        Err(e) => error!("Error during MQTT client disconnect: {:?}", e),
    }
}
