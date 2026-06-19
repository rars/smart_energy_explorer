use std::{pin::Pin, time::Duration};

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
    pub energy: ElectricityEnergyContainer,
    pub power: Power,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ElectricityEnergyContainer {
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GasMeter {
    pub timestamp: String,
    pub energy: GasEnergyContainer,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GasEnergyContainer {
    pub import: GasImport,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct GasImport {
    pub cumulative: f64,
    pub day: f64,
    pub week: f64,
    pub month: f64,
    pub units: String,

    pub cumulativevol: f64,
    pub cumulativevolunits: String,

    pub dayvol: f64,
    pub weekvol: f64,
    pub monthvol: f64,
    pub dayweekmonthvolunits: String,

    pub mprn: String,
    pub supplier: String,
    pub price: ImportPrice,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MeterPayload {
    ElectricityMeter(ElectricityMeter),
    GasMeter(GasMeter),
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GasMeterMessage {
    pub gasmeter: GasMeter,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GasUpdate {
    is_active: bool,
    message: GasMeterMessage,
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

    if settings.topic.len() > 0 {
        client.subscribe(settings.topic.clone(), qos).await?;
    }

    if settings.gas_topic.len() > 0 {
        client.subscribe(settings.gas_topic.clone(), qos).await?;
    }

    Ok(client)
}

enum MqttClientState {
    Disconnected,
    Connected(AsyncClient, Pin<Box<AsyncReceiver<Option<Message>>>>),
}

pub async fn start_mqtt_listener(
    app_handle: &AppHandle,
    mut mqtt_message_receiver: mpsc::Receiver<MqttMessage>,
) {
    info!("Starting MQTT listener thread.");

    let uuid_string = Uuid::new_v4().to_string();
    let client_id = format!("smart-energy-explorer-{}", uuid_string);
    info!("Generated Client ID: {}", client_id);

    let mut mqtt_client_state = MqttClientState::Disconnected;

    loop {
        let (client, stream) = match &mut mqtt_client_state {
            MqttClientState::Connected(c, s) => (c, s),
            MqttClientState::Disconnected => {
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
                                let stream = Box::pin(client.get_stream(None));
                                info!("MQTT client and stream created");
                                mqtt_client_state = MqttClientState::Connected(client, stream);
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

                // Sleep to avoid tight loop, but settings updates should be handled immediately
                tokio::select! {
                  Some(app_message) = mqtt_message_receiver.recv() => {
                      match app_message {
                          MqttMessage::SettingsUpdated => {
                              info!("MQTT settings updated");
                          }
                      }
                  },
                  _ = tokio::time::sleep(Duration::from_secs(10)) => {}
                }

                continue;
            }
        };

        if !client.is_connected() {
            info!("The MQTT client is no longer connected. Resetting client and stream.");
            mqtt_client_state = MqttClientState::Disconnected;
            continue;
        }

        tokio::select! {
            Some(app_message) = mqtt_message_receiver.recv() => {
                match app_message {
                    MqttMessage::SettingsUpdated => {
                        info!("MQTT settings updated");
                        if let MqttClientState::Connected(client, _) = &mqtt_client_state {
                          info!("Disconnecting existing MQTT client due to settings update...");
                          if client.is_connected() {
                            disconnect_client(client).await;
                          }
                        }
                        mqtt_client_state = MqttClientState::Disconnected;
                    }
                }
            },
            message = stream.next() => {
                match message {
                    Some(Some(msg)) => {
                        match serde_json::from_str::<MeterPayload>(&msg.payload_str()) {
                            Ok(payload) => {
                                info!("Deserialized data: {:?}", payload);
                                // Now you can work with the structured 'data' object

                                match payload {
                                    MeterPayload::ElectricityMeter(data) => {
                                      if let Err(err) = emit_event(app_handle, "electricityUpdate", ElectricityMeterMessage { electricitymeter: data }) {
                                          error!("Unexpected error emitting electricityUpdate event: {}", err);
                                      }
                                    },
                                    MeterPayload::GasMeter(data) => {
                                      if let Err(err) = emit_event(app_handle, "gasUpdate", GasMeterMessage { gasmeter: data }) {
                                          error!("Unexpected error emitting gasUpdate event: {}", err);
                                      }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to deserialize payload: {}", e);
                            }
                        }
                    }
                    Some(None) => {
                        info!("Unexpected message from MQTT stream receiever: None");
                    }
                    None => {
                        info!("Notified that the MQTT client connection has been lost. Resetting client and stream.");
                        if client.is_connected() {
                            disconnect_client(&client).await;
                        }
                        mqtt_client_state = MqttClientState::Disconnected;
                    },
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
