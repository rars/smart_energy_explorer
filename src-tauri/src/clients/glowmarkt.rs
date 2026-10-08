use rust_decimal::{prelude::FromPrimitive, Decimal};
use std::{collections::HashMap, sync::Arc};
use tauri::async_runtime::Mutex;

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use glowmarkt::{ErrorKind, GlowmarktApi, ReadingPeriod};
use time::{
    error::ComponentRange, macros::date, macros::time, Date, OffsetDateTime, PrimitiveDateTime,
    Time,
};

use crate::data::{
    consumption::{ElectricityConsumptionValue, GasConsumptionValue},
    tariff::TariffPlan,
};

use super::data_provider::EnergyDataProvider;
use crate::retry::with_retry;

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

#[derive(Debug)]
struct ResourceIds {
    gas_cost: Option<String>,
    gas_consumption: Option<String>,
    electricity_cost: Option<String>,
    electricity_consumption: Option<String>,
}

async fn get_resource_ids(api: &GlowmarktApi) -> Result<ResourceIds, GlowmarktDataProviderError> {
    let all_resources = api
        .resources()
        .await
        .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

    let virtual_entities = api
        .virtual_entities()
        .await
        .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

    extract_resource_ids(&all_resources, &virtual_entities)
}

fn extract_resource_ids(
    all_resources: &HashMap<String, glowmarkt::Resource>,
    virtual_entities: &HashMap<String, glowmarkt::VirtualEntity>,
) -> Result<ResourceIds, GlowmarktDataProviderError> {
    let mut resource_ids = ResourceIds {
        gas_cost: None,
        gas_consumption: None,
        electricity_cost: None,
        electricity_consumption: None,
    };

    for virtual_entity in virtual_entities.values() {
        if virtual_entity.name == "DCC Sourced" {
            for resource_info in &virtual_entity.resources {
                if let Some(resource) = all_resources.get(&resource_info.resource_id) {
                    match resource.classifier.as_ref().map(|c| c.as_str()) {
                        Some("electricity.consumption") => {
                            resource_ids.electricity_consumption = Some(resource.id.clone())
                        }
                        Some("electricity.consumption.cost") => {
                            resource_ids.electricity_cost = Some(resource.id.clone())
                        }
                        Some("gas.consumption") => {
                            resource_ids.gas_consumption = Some(resource.id.clone())
                        }
                        Some("gas.consumption.cost") => {
                            resource_ids.gas_cost = Some(resource.id.clone())
                        }
                        _ => (),
                    };
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
                self.get_time_month(&start),
                start.day0() as u8 + 1u8,
            )
            .map_err(|e| GlowmarktDataProviderError::TimeComponentRangeError(e))?,
            Time::from_hms(0, 0, 0)
                .map_err(|e| GlowmarktDataProviderError::TimeComponentRangeError(e))?,
        );

        let to: OffsetDateTime = OffsetDateTime::new_utc(
            Date::from_calendar_date(
                end.year(),
                self.get_time_month(&end),
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

    fn get_time_month(&self, date: &NaiveDate) -> time::Month {
        time::Month::try_from(date.month() as u8)
            .expect("Failed to extract valid month from NaiveDate")
    }
}

impl EnergyDataProvider for GlowmarktDataProvider {
    type Error = GlowmarktDataProviderError;

    fn has_electricity_consumption(&self) -> bool {
        self.resource_ids.electricity_consumption.is_some()
    }

    async fn get_electricity_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<ElectricityConsumptionValue>, Self::Error> {
        if let Some(resource_id) = &self.resource_ids.electricity_consumption {
            let offset_date_range = self.get_range(start, end)?;

            let response = with_retry(
                async || {
                    let api = self.api.lock().await;
                    api.readings(
                        resource_id,
                        &offset_date_range.start,
                        &offset_date_range.end,
                        ReadingPeriod::HalfHour,
                    )
                    .await
                },
                is_retryable,
                3,
            )
            .await;

            let readings = response
                .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

            let consumption_values: Vec<_> = readings
                .iter()
                .map(|v| ElectricityConsumptionValue {
                    timestamp: to_naive_date_time(v.start),
                    value: Decimal::from_f64(v.value).expect("f64 should fit into Decimal"),
                })
                .collect();

            return Ok(consumption_values);
        }

        Err(GlowmarktDataProviderError::MissingResource(
            "electricity consumption".to_string(),
        ))
    }

    fn has_electricity_tariff_history(&self) -> bool {
        self.resource_ids.electricity_cost.is_some()
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

    fn has_gas_consumption(&self) -> bool {
        self.resource_ids.gas_consumption.is_some()
    }

    async fn get_gas_consumption(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<GasConsumptionValue>, Self::Error> {
        if let Some(resource_id) = &self.resource_ids.gas_consumption {
            let offset_date_range = self.get_range(start, end)?;

            let response = with_retry(
                async || {
                    let api = self.api.lock().await;
                    api.readings(
                        resource_id,
                        &offset_date_range.start,
                        &offset_date_range.end,
                        ReadingPeriod::HalfHour,
                    )
                    .await
                },
                is_retryable,
                3,
            )
            .await;

            let readings = response
                .map_err(|e| GlowmarktDataProviderError::GlowmarktApiError(e.to_string()))?;

            let consumption_values: Vec<_> = readings
                .iter()
                .map(|v| GasConsumptionValue {
                    timestamp: to_naive_date_time(v.start),
                    value: Decimal::from_f64(v.value).expect("f64 should fit into Decimal"),
                })
                .collect();

            return Ok(consumption_values);
        }

        Err(GlowmarktDataProviderError::MissingResource(
            "gas consumption".to_string(),
        ))
    }

    fn has_gas_tariff_history(&self) -> bool {
        self.resource_ids.gas_cost.is_some()
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

fn is_retryable(error: &glowmarkt::Error) -> bool {
    matches!(error.kind, ErrorKind::Server | ErrorKind::Network)
}

#[cfg(test)]
mod tests {
    use super::*;
    use glowmarkt::{api::ResourceInfo, Resource, VirtualEntity};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_is_retryable_returns_true_for_server_error() {
        let server_error = glowmarkt::Error {
            kind: glowmarkt::ErrorKind::Server,
            message: "Server error".to_string(),
        };
        assert!(is_retryable(&server_error));
    }

    #[test]
    fn test_is_retryable_returns_true_for_network_error() {
        let network_error = glowmarkt::Error {
            kind: glowmarkt::ErrorKind::Network,
            message: "Network error".to_string(),
        };
        assert!(is_retryable(&network_error));
    }

    #[test]
    fn test_is_retryable_returns_false_for_not_authenticated_error() {
        let unauthenticated_error = glowmarkt::Error {
            kind: glowmarkt::ErrorKind::NotAuthenticated,
            message: "Other error".to_string(),
        };
        assert!(!is_retryable(&unauthenticated_error));
    }

    #[test]
    fn test_primitive_to_naive_date_time() {
        let primitive_dt = PrimitiveDateTime::new(date!(2024 - 06 - 01), time!(12:30:45));
        let naive_dt = primitive_to_naive_date_time(primitive_dt);
        assert_eq!(
            naive_dt,
            NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(12, 30, 45).unwrap()
            )
        );
    }

    #[test]
    fn test_offset_to_naive_date_time() {
        let offset_dt = OffsetDateTime::new_utc(
            Date::from_calendar_date(2024, time::Month::June, 1).unwrap(),
            Time::from_hms(12, 30, 45).unwrap(),
        );
        let naive_dt = to_naive_date_time(offset_dt);
        assert_eq!(
            naive_dt,
            NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2024, 6, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(12, 30, 45).unwrap()
            )
        );
    }

    fn resource(id: &str, classifier: Option<&str>) -> (String, Resource) {
        (
            id.to_string(),
            Resource {
                id: id.to_string(),
                name: format!("Resource {id}"),
                description: None,
                label: None,
                active: true,
                type_id: "t".to_string(),
                owner_id: "o".to_string(),
                classifier: classifier.map(str::to_string),
                base_unit: None,
                data_source_type: "DCC".to_string(),
                data_source_resource_type_info: None,
                data_source_unit_info: None,
                updated_at: OffsetDateTime::UNIX_EPOCH,
                created_at: OffsetDateTime::UNIX_EPOCH,
            },
        )
    }

    fn entity(name: &str, resource_ids: &[&str]) -> VirtualEntity {
        VirtualEntity {
            id: format!("entity_{name}"),
            name: name.to_string(),
            active: true,
            type_id: "t".to_string(),
            owner_id: "o".to_string(),
            resources: resource_ids
                .iter()
                .map(|&id| ResourceInfo {
                    resource_id: id.to_string(),
                    resource_type_id: "t".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn test_extract_resource_ids() -> Result<(), GlowmarktDataProviderError> {
        let resources = HashMap::from([
            resource("res1", Some("electricity.consumption")),
            resource("res2", Some("electricity.consumption.cost")),
            resource("res3", Some("gas.consumption")),
            resource("res4", Some("gas.consumption.cost")),
        ]);

        let entities = HashMap::from([(
            "entity1".to_string(),
            entity("DCC Sourced", &["res1", "res2", "res3", "res4"]),
        )]);

        let ids = extract_resource_ids(&resources, &entities)?;

        assert_eq!(ids.electricity_consumption, Some("res1".to_string()));
        assert_eq!(ids.electricity_cost, Some("res2".to_string()));
        assert_eq!(ids.gas_consumption, Some("res3".to_string()));
        assert_eq!(ids.gas_cost, Some("res4".to_string()));

        Ok(())
    }
}
