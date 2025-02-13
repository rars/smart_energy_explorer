use log::info;
use std::sync::Arc;
use tauri::async_runtime::Mutex;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use glowmarkt::{GlowmarktApi, ReadingPeriod};
use time::{
    error::ComponentRange, macros::date, macros::time, Date, OffsetDateTime, PrimitiveDateTime,
    Time,
};

use crate::data::{
    consumption::{ElectricityConsumptionValue, GasConsumptionValue},
    tariff::TariffPlan,
};

use super::data_provider::EnergyDataProvider;

struct OffsetDateTimeRange {
    start: OffsetDateTime,
    end: OffsetDateTime,
}

#[derive(Debug, thiserror::Error)]
pub enum GlowmarktDataProviderError {
    #[error("Failed request to Glowmarkt API: {0}")]
    GlowmarktApiError(String),
    #[error("Time component range error: {0}")]
    TimeComponentRangeError(#[from] ComponentRange),
    #[error("Missing resource: {0}")]
    MissingResource(String),
}

struct ResourceIds {
    gas_cost: Option<String>,
    gas_consumption: Option<String>,
    electricity_cost: Option<String>,
    electricity_consumption: Option<String>,
}

async fn get_resource_ids(api: &GlowmarktApi) -> Result<ResourceIds, GlowmarktDataProviderError> {
    let mut resource_ids = ResourceIds {
        gas_cost: None,
        gas_consumption: None,
        electricity_cost: None,
        electricity_consumption: None,
    };

    let all_resources = api
        .resources()
        .await
        .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

    let virtual_entities = api
        .virtual_entities()
        .await
        .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

    for virtual_entity in virtual_entities.values() {
        if virtual_entity.name == "DCC Sourced" {
            for resource_info in &virtual_entity.resources {
                if let Some(resource) = all_resources.get(&resource_info.resource_id) {
                    match resource.name.as_str() {
                        "electricity cost" => {
                            resource_ids.electricity_cost = Some(resource.id.clone())
                        }
                        "electricity consumption" => {
                            resource_ids.electricity_consumption = Some(resource.id.clone())
                        }
                        "gas cost" => resource_ids.gas_cost = Some(resource.id.clone()),
                        "gas consumption" => {
                            resource_ids.gas_consumption = Some(resource.id.clone())
                        }
                        _ => (),
                    }
                }
            }
            break;
        }
    }

    Ok(resource_ids)
}

fn to_naive_date_time(offset_dt: OffsetDateTime) -> NaiveDateTime {
    NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(
            offset_dt.year(),
            offset_dt.month() as u32,
            offset_dt.day().into(),
        )
        .unwrap(),
        chrono::NaiveTime::from_hms_opt(
            offset_dt.hour().into(),
            offset_dt.minute().into(),
            offset_dt.second().into(),
        )
        .unwrap(),
    )
}

fn primitive_to_naive_date_time(primitive_dt: PrimitiveDateTime) -> NaiveDateTime {
    NaiveDateTime::new(
        chrono::NaiveDate::from_ymd_opt(
            primitive_dt.year(),
            primitive_dt.month() as u32,
            primitive_dt.day().into(),
        )
        .unwrap(),
        chrono::NaiveTime::from_hms_opt(
            primitive_dt.hour().into(),
            primitive_dt.minute().into(),
            primitive_dt.second().into(),
        )
        .unwrap(),
    )
}

pub struct GlowmarktDataProvider {
    api: Arc<Mutex<GlowmarktApi>>,
    resource_ids: ResourceIds,
}

impl GlowmarktDataProvider {
    pub async fn new(username: &str, password: &str) -> Result<Self, GlowmarktDataProviderError> {
        let api = GlowmarktApi::authenticate(username, password)
            .await
            .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

        let resource_ids = get_resource_ids(&api).await?;

        info!("~~~~~~~~~~~~~~ Created Glowmarkt Data Provider!!! ~~~~~~~~~~~~~~~~~");

        Ok(Self {
            api: Arc::new(Mutex::new(api)),
            resource_ids,
        })
    }

    fn get_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<OffsetDateTimeRange, GlowmarktDataProviderError> {
        let from = OffsetDateTime::new_utc(
            Date::from_calendar_date(
                start.year(),
                self.get_month(start.month()),
                start.day0() as u8 + 1u8,
            )
            .map_err(|e| GlowmarktDataProviderError::TimeComponentRangeError(e))?,
            Time::from_hms(0, 0, 0)
                .map_err(|e| GlowmarktDataProviderError::TimeComponentRangeError(e))?,
        );

        let to: OffsetDateTime = OffsetDateTime::new_utc(
            Date::from_calendar_date(
                end.year(),
                self.get_month(end.month()),
                end.day0() as u8 + 1u8,
            )
            .map_err(|e| GlowmarktDataProviderError::TimeComponentRangeError(e))?,
            Time::from_hms(0, 0, 0)
                .map_err(|e| GlowmarktDataProviderError::TimeComponentRangeError(e))?,
        );

        Ok(OffsetDateTimeRange {
            start: from,
            end: to,
        })
    }

    fn get_month(&self, month: u32) -> time::Month {
        match month {
            1 => time::Month::January,
            2 => time::Month::February,
            3 => time::Month::March,
            4 => time::Month::April,
            5 => time::Month::May,
            6 => time::Month::June,
            7 => time::Month::July,
            8 => time::Month::August,
            9 => time::Month::September,
            10 => time::Month::October,
            11 => time::Month::November,
            12 => time::Month::December,
            _ => time::Month::January,
        }
    }
}

impl EnergyDataProvider for GlowmarktDataProvider {
    type Error = GlowmarktDataProviderError;

    async fn get_electricity_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityConsumptionValue>, Self::Error> {
        if let Some(resource_id) = &self.resource_ids.electricity_consumption {
            let offset_date_range = self.get_range(start, end)?;

            let readings = {
                let api = self.api.lock().await;
                api.readings(
                    resource_id,
                    &offset_date_range.start,
                    &offset_date_range.end,
                    ReadingPeriod::HalfHour,
                )
                .await
            }
            .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

            let consumption_values: Vec<_> = readings
                .iter()
                .map(|v| ElectricityConsumptionValue {
                    timestamp: to_naive_date_time(v.start),
                    value: v.value as f64,
                })
                .collect();

            return Ok(consumption_values);
        }

        Err(GlowmarktDataProviderError::MissingResource(
            "electricity consumption".to_string(),
        ))
    }

    async fn get_electricity_tariff_history(
        &self,
    ) -> Result<Vec<crate::data::tariff::TariffPlan>, Self::Error> {
        if let Some(resource_id) = &self.resource_ids.electricity_cost {
            let tariff_list_data = {
                let api = self.api.lock().await;
                api.tariff_list(resource_id).await
            }
            .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

            let consumption_values: Vec<_> = tariff_list_data
                .into_iter()
                .map(|v| TariffPlan {
                    tariff_id: v.id,
                    plan: serde_json::to_string(&v.plan).unwrap(),
                    effective_date: primitive_to_naive_date_time(
                        v.effective_date
                            .or(v.from)
                            .unwrap_or(PrimitiveDateTime::new(date!(1900 - 01 - 01), time!(0:00))),
                    ),
                    display_name: v.display_name.unwrap_or("<unknown>".into()),
                })
                .collect();

            return Ok(consumption_values);
        }

        Err(GlowmarktDataProviderError::MissingResource(
            "electricity cost".to_string(),
        ))
    }

    async fn get_gas_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasConsumptionValue>, Self::Error> {
        if let Some(resource_id) = &self.resource_ids.gas_consumption {
            let offset_date_range = self.get_range(start, end)?;

            let readings = {
                let api = self.api.lock().await;
                api.readings(
                    resource_id,
                    &offset_date_range.start,
                    &offset_date_range.end,
                    ReadingPeriod::HalfHour,
                )
                .await
            }
            .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

            let consumption_values: Vec<_> = readings
                .iter()
                .map(|v| GasConsumptionValue {
                    timestamp: to_naive_date_time(v.start),
                    value: v.value as f64,
                })
                .collect();

            return Ok(consumption_values);
        }

        Err(GlowmarktDataProviderError::MissingResource(
            "gas consumption".to_string(),
        ))
    }

    async fn get_gas_tariff_history(
        &self,
    ) -> Result<Vec<crate::data::tariff::TariffPlan>, Self::Error> {
        if let Some(resource_id) = &self.resource_ids.gas_cost {
            let tariff_list_data = {
                let api = self.api.lock().await;
                api.tariff_list(resource_id).await
            }
            .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

            let consumption_values: Vec<_> = tariff_list_data
                .into_iter()
                .map(|v| TariffPlan {
                    tariff_id: v.id,
                    plan: serde_json::to_string(&v.plan).unwrap(),
                    effective_date: primitive_to_naive_date_time(
                        v.effective_date
                            .or(v.from)
                            .unwrap_or(PrimitiveDateTime::new(date!(1900 - 01 - 01), time!(0:00))),
                    ),
                    display_name: v.display_name.unwrap_or("<unknown>".into()),
                })
                .collect();

            return Ok(consumption_values);
        }

        Err(GlowmarktDataProviderError::MissingResource(
            "gas cost".to_string(),
        ))
    }
}
