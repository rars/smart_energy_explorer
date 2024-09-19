use chrono::{NaiveDate, NaiveDateTime};
use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StandingCharge {
    pub start_date: NaiveDateTime,
    pub standing_charge_pence: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UnitPrice {
    pub price_effective_time: NaiveDateTime,
    pub unit_price_pence: f64,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TariffHistoryResponse {
    pub standing_charges: Vec<StandingCharge>,
    pub unit_prices: Vec<UnitPrice>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DailyCost {
    pub date: NaiveDate,
    pub cost_pence: f64,
}
