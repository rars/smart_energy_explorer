use std::error::Error;

use chrono::NaiveDate;

use crate::data::{
    consumption::{ElectricityConsumptionValue, GasConsumptionValue},
    tariff::TariffPlan,
};

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

    fn get_gas_tariff_history(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TariffPlan>, Self::Error>> + Send;

    fn get_electricity_tariff_history(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TariffPlan>, Self::Error>> + Send;
}
