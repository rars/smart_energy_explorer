use std::{
    cmp,
    error::Error,
    future::Future,
    sync::{Arc, Mutex},
};

use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use diesel::SqliteConnection;
use log::{debug, error, info};
use serde::Serialize;
use tauri::{async_runtime, AppHandle};

use crate::{
    clients::data_provider::EnergyDataProvider,
    data::{
        consumption::{
            ConsumptionRepository, ElectricityConsumptionValue, GasConsumptionValue,
            SqliteElectricityConsumptionRepository, SqliteGasConsumptionRepository,
        },
        energy_profile::{EnergyProfileRepository, SqliteEnergyProfileRepository},
        tariff::{
            NewElectricityTariffPlan, NewGasTariffPlan, SqliteElectricityTariffRepository,
            SqliteGasTariffRepository, TariffPlan, TariffRepository,
        },
        RepositoryError,
    },
    utils::{emit_event, get_or_create_energy_profile},
    AppError, AppState,
};

#[derive(Serialize, Clone)]
struct DownloadUpdateEvent {
    percentage: u32,
    name: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AppStatusUpdateEvent {
    pub is_downloading: bool,
}

pub trait DataLoader<T> {
    type LoadError: Error + Send + Sync + 'static;
    type InsertError: Error + Send + Sync + 'static;

    async fn load(&self, start: NaiveDate, end: NaiveDate) -> Result<Vec<T>, Self::LoadError>;
    fn insert_data(&self, data: Vec<T>) -> Result<(), Self::InsertError>;
}

#[derive(Clone)]
struct ElectricityConsumptionDataLoader<T>
where
    T: EnergyDataProvider,
{
    data_provider: Arc<T>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<ElectricityConsumptionValue> for ElectricityConsumptionDataLoader<T>
where
    T: EnergyDataProvider,
{
    type LoadError = T::Error;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityConsumptionValue>, Self::LoadError> {
        let values = self
            .data_provider
            .get_electricity_consumption(start, end)
            .await?;
        Ok(values)
    }

    fn insert_data(&self, data: Vec<ElectricityConsumptionValue>) -> Result<(), Self::InsertError> {
        SqliteElectricityConsumptionRepository::new(self.connection.clone()).insert(data)?;

        Ok(())
    }
}

#[derive(Clone)]
struct ElectricityTariffDataLoader<T>
where
    T: EnergyDataProvider,
{
    data_provider: Arc<T>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<TariffPlan> for ElectricityTariffDataLoader<T>
where
    T: EnergyDataProvider,
{
    type LoadError = T::Error;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<TariffPlan>, Self::LoadError> {
        Ok(self.data_provider.get_electricity_tariff_history().await?)
    }

    fn insert_data(&self, data: Vec<TariffPlan>) -> Result<(), Self::InsertError> {
        let tps: Vec<_> = data
            .into_iter()
            .map(|tp| NewElectricityTariffPlan {
                tariff_id: tp.tariff_id,
                plan: tp.plan,
                effective_date: tp.effective_date,
                display_name: tp.display_name,
            })
            .collect();

        SqliteElectricityTariffRepository::new(self.connection.clone()).insert(tps)?;

        Ok(())
    }
}

#[derive(Clone)]
struct GasConsumptionDataLoader<T>
where
    T: EnergyDataProvider,
{
    data_provider: Arc<T>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<GasConsumptionValue> for GasConsumptionDataLoader<T>
where
    T: EnergyDataProvider,
{
    type LoadError = T::Error;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasConsumptionValue>, Self::LoadError> {
        let values = self.data_provider.get_gas_consumption(start, end).await?;
        Ok(values)
    }

    fn insert_data(&self, data: Vec<GasConsumptionValue>) -> Result<(), Self::InsertError> {
        SqliteGasConsumptionRepository::new(self.connection.clone()).insert(data)?;
        Ok(())
    }
}

#[derive(Clone)]
struct GasTariffDataLoader<T>
where
    T: EnergyDataProvider,
{
    data_provider: Arc<T>,
    connection: Arc<Mutex<SqliteConnection>>,
}

impl<T> DataLoader<TariffPlan> for GasTariffDataLoader<T>
where
    T: EnergyDataProvider,
{
    type LoadError = T::Error;
    type InsertError = RepositoryError;

    async fn load(
        &self,
        _start: NaiveDate,
        _end: NaiveDate,
    ) -> Result<Vec<TariffPlan>, Self::LoadError> {
        Ok(self.data_provider.get_gas_tariff_history().await?)
    }

    fn insert_data(&self, data: Vec<TariffPlan>) -> Result<(), Self::InsertError> {
        let tps: Vec<_> = data
            .into_iter()
            .map(|tp| NewGasTariffPlan {
                tariff_id: tp.tariff_id,
                plan: tp.plan,
                effective_date: tp.effective_date,
                display_name: tp.display_name,
            })
            .collect();

        SqliteGasTariffRepository::new(self.connection.clone()).insert(tps)?;

        Ok(())
    }
}

pub async fn download_history<T, U>(
    app_handle: AppHandle,
    data_loader: T,
    until_date_time: NaiveDateTime,
    download_name: &str,
) -> Result<NaiveDate, AppError>
where
    T: DataLoader<U>,
    T::LoadError: Error + Send + Sync + 'static,
{
    let until_date = until_date_time.date();

    let today = Local::now().naive_local().date();
    let mut end_date = today;
    let mut start_of_period = today - Duration::days(7);

    if start_of_period < until_date {
        start_of_period = until_date;
    }

    let total_days = today.signed_duration_since(until_date).num_days();

    while start_of_period >= until_date && start_of_period < end_date {
        let records = data_loader
            .load(start_of_period, end_date)
            .await
            .map_err(|e| AppError::CustomError(format!("Error while loading data: {}", e)))?;

        info!(
            "For {} to {}, inserting {} records.",
            start_of_period,
            end_date,
            records.len()
        );

        if records.len() > 0 {
            data_loader
                .insert_data(records)
                .map_err(|e| AppError::CustomError(format!("Error while inserting data: {}", e)))?;
        }

        end_date = start_of_period;
        // For Glowmarkt API on 30 minute intervals, you can request max 10 days of data at a time.
        start_of_period = start_of_period - Duration::days(7);

        if start_of_period < until_date {
            start_of_period = until_date;
        }

        let days_remaining = end_date.signed_duration_since(until_date).num_days();

        let percentage = 100.0 * (1.0 - (days_remaining as f64 / total_days as f64));

        emit_event(
            &app_handle,
            "downloadUpdate",
            DownloadUpdateEvent {
                percentage: percentage.round() as u32,
                name: download_name.into(),
            },
        )?;
    }

    emit_event(
        &app_handle,
        "downloadUpdate",
        DownloadUpdateEvent {
            percentage: 100,
            name: download_name.into(),
        },
    )?;

    Ok(today)
}

pub async fn check_and_download_new_data<U>(
    app_handle: AppHandle,
    app_state: AppState,
    data_provider: Arc<U>,
) -> Result<(), AppError>
where
    U: EnergyDataProvider,
{
    {
        let mut downloading = app_state
            .downloading
            .lock()
            .map_err(|e| AppError::CustomError(format!("Failed to acquire lock, error: {}", e)))?;

        if *downloading {
            return Ok(());
        }

        *downloading = true;
    }

    emit_event(
        &app_handle,
        "appStatusUpdate",
        AppStatusUpdateEvent {
            is_downloading: true,
        },
    )?;

    let electricity_consumption_data_loader = ElectricityConsumptionDataLoader {
        data_provider: data_provider.clone(),
        connection: app_state.db.clone(),
    };

    let electricity_tariff_data_loader = ElectricityTariffDataLoader {
        data_provider: data_provider.clone(),
        connection: app_state.db.clone(),
    };

    let app_handle_clone = app_handle.clone();

    check_for_new_data(
        app_state.db.clone(),
        "electricity",
        "kWh",
        move |until_date_time| async move {
            let date_one = download_history(
                app_handle_clone.clone(),
                electricity_consumption_data_loader,
                until_date_time,
                "electricity consumption",
            )
            .await?;

            let date_two = download_history(
                app_handle_clone,
                electricity_tariff_data_loader,
                until_date_time,
                "electricity tariff",
            )
            .await?;

            Ok(cmp::max(date_one, date_two))
        },
    )
    .await?;

    let gas_consumption_data_loader = GasConsumptionDataLoader {
        data_provider: data_provider.clone(),
        connection: app_state.db.clone(),
    };

    let gas_tariff_data_loader = GasTariffDataLoader {
        data_provider: data_provider.clone(),
        connection: app_state.db.clone(),
    };

    let app_handle_clone = app_handle.clone();

    check_for_new_data(
        app_state.db.clone(),
        "gas",
        "kWh",
        move |until_date_time| async move {
            let date_one = download_history(
                app_handle_clone.clone(),
                gas_consumption_data_loader,
                until_date_time,
                "gas consumption",
            )
            .await?;

            let date_two = download_history(
                app_handle_clone,
                gas_tariff_data_loader,
                until_date_time,
                "gas tariff",
            )
            .await?;

            Ok(cmp::max(date_one, date_two))
        },
    )
    .await?;

    {
        let mut downloading = app_state
            .downloading
            .lock()
            .map_err(|e| AppError::CustomError(format!("Failed to acquire lock, error: {}", e)))?;

        *downloading = false;
    }

    emit_event(
        &app_handle,
        "appStatusUpdate",
        AppStatusUpdateEvent {
            is_downloading: false,
        },
    )?;

    Ok(())
}

async fn check_for_new_data<F, Fut>(
    connection: Arc<Mutex<SqliteConnection>>,
    profile_name: &str,
    base_unit: &str,
    download_action: F,
) -> Result<(), AppError>
where
    F: FnOnce(NaiveDateTime) -> Fut,
    Fut: Future<Output = Result<NaiveDate, AppError>>,
{
    let profile = get_or_create_energy_profile(connection.clone(), profile_name, base_unit)?;

    if !profile.is_active {
        info!(
            "{} profile is not active. Will not download historical data.",
            profile_name
        );
        return Ok(());
    }

    let until_date_time = profile.last_date_retrieved.unwrap_or(profile.start_date);

    let last_date_retrieved = download_action(until_date_time).await?;

    let repository = SqliteEnergyProfileRepository::new(connection);

    repository
        .update_energy_profile(
            profile.energy_profile_id,
            profile.is_active,
            profile.start_date,
            last_date_retrieved.into(),
        )
        .map_err(|error| {
            AppError::CustomError(format!(
                "Failed to update {} consumption profile, error: {}",
                profile_name, error
            ))
        })?;

    info!("Successfully updated {} consumption profile", profile_name);

    Ok(())
}

pub fn spawn_download_tasks<T>(
    app_handle: AppHandle,
    app_state: AppState,
    data_provider: T,
) -> Result<(), AppError>
where
    T: EnergyDataProvider + 'static,
{
    info!("Spawning download tasks");
    let data_provider = Arc::new(data_provider);

    async_runtime::spawn(async move {
        match check_and_download_new_data(app_handle, app_state, data_provider).await {
            Ok(_) => debug!("Data download tasks completed successfully"),
            Err(e) => {
                error!("Data download tasks panicked: {:?}", e);
                // Handle the panic (e.g., restart the task, log the error, etc.)
            }
        }
    });

    Ok(())
}
