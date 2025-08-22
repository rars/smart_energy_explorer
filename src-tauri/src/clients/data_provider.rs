use std::error::Error;

use chrono::NaiveDate;

use crate::data::{
    consumption::{ElectricityConsumptionValue, GasConsumptionValue},
    tariff::TariffPlan,
};

pub trait EnergyDataProvider: Send + Sync {
    type Error: Error + Send + Sync + 'static;

    fn has_electricity_consumption(&self) -> bool;

    fn get_electricity_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl std::future::Future<Output = Result<Vec<ElectricityConsumptionValue>, Self::Error>> + Send;

    fn has_gas_consumption(&self) -> bool;

    fn get_gas_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> impl std::future::Future<Output = Result<Vec<GasConsumptionValue>, Self::Error>> + Send;

    fn has_gas_tariff_history(&self) -> bool;

    fn get_gas_tariff_history(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TariffPlan>, Self::Error>> + Send;

    fn has_electricity_tariff_history(&self) -> bool;

    fn get_electricity_tariff_history(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<TariffPlan>, Self::Error>> + Send;
}
