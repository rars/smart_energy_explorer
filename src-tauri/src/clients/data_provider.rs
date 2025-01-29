use std::error::Error;

use chrono::NaiveDate;

use crate::data::consumption::{ElectricityConsumptionValue, GasConsumptionValue};

pub trait EnergyDataProvider: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    fn get_electricity_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl std::future::Future<Output = Result<Vec<ElectricityConsumptionValue>, Self::Error>> + Send;

    fn get_gas_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl std::future::Future<Output = Result<Vec<GasConsumptionValue>, Self::Error>> + Send;
}
