use std::sync::Arc;

use chrono::NaiveDate;
use n3rgy_consumer_api_client::{AuthorizationProvider, ConsumerApiClient, N3rgyClientError};

use crate::data::consumption::{ElectricityConsumptionValue, GasConsumptionValue};

use super::data_provider::EnergyDataProvider;

#[derive(Debug, thiserror::Error)]
pub enum N3rgyDataProviderError {
    #[error("Failed request to n3rgy API: {0}")]
    N3rgyClientError(#[from] N3rgyClientError),
}

#[derive(Clone)]
pub struct N3rgyEnergyDataProvider<T>
where
    T: AuthorizationProvider,
{
    client: Arc<ConsumerApiClient<T>>,
}

impl<T> N3rgyEnergyDataProvider<T>
where
    T: AuthorizationProvider,
{
    pub fn new(client: Arc<ConsumerApiClient<T>>) -> Self {
        Self { client }
    }
}

impl<T> EnergyDataProvider for N3rgyEnergyDataProvider<T>
where
    T: Send + Sync + AuthorizationProvider,
{
    type Error = N3rgyDataProviderError;

    async fn get_electricity_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityConsumptionValue>, Self::Error> {
        let readings = self
            .client
            .get_electricity_consumption(start, end)
            .await
            .map_err(|e| N3rgyDataProviderError::N3rgyClientError(e))?;

        let consumption_values: Vec<_> = readings
            .iter()
            .map(|v| ElectricityConsumptionValue {
                timestamp: v.timestamp,
                value: v.value,
            })
            .collect();

        Ok(consumption_values)
    }

    async fn get_gas_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasConsumptionValue>, Self::Error> {
        let readings = self
            .client
            .get_gas_consumption(start, end)
            .await
            .map_err(|e| N3rgyDataProviderError::N3rgyClientError(e))?;

        let consumption_values: Vec<_> = readings
            .iter()
            .map(|v| GasConsumptionValue {
                timestamp: v.timestamp,
                value: v.value,
            })
            .collect();

        Ok(consumption_values)
    }
}
