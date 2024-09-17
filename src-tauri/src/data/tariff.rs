use std::sync::{Arc, Mutex, MutexGuard};

use chrono::NaiveDateTime;
use diesel::sql_types::{Double, Timestamp};
use diesel::{insert_into, sql_query, Queryable, SqliteConnection};
use diesel::{prelude::*, upsert::excluded};
use n3rgy_consumer_api_client::{ElectricityTariff, GasTariff};

use super::RepositoryError;
use crate::schema::{
    electricity_standing_charge, electricity_unit_price, gas_standing_charge, gas_unit_price,
};

#[derive(Insertable)]
#[diesel(table_name = electricity_standing_charge)]
struct NewElectricityStandingCharge {
    start_date: NaiveDateTime,
    standing_charge_pence: f64,
}

#[derive(Insertable)]
#[diesel(table_name = gas_standing_charge)]
struct NewGasStandingCharge {
    start_date: NaiveDateTime,
    standing_charge_pence: f64,
}

#[derive(Insertable)]
#[diesel(table_name = electricity_unit_price)]
struct NewElectricityUnitPrice {
    price_effective_time: NaiveDateTime,
    unit_price_pence: f64,
}

#[derive(Insertable)]
#[diesel(table_name = gas_unit_price)]
struct NewGasUnitPrice {
    price_effective_time: NaiveDateTime,
    unit_price_pence: f64,
}

#[derive(Queryable)]
pub struct ElectricityTariffRecord {
    pub electricity_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_kwh: f64,
}

#[derive(Queryable)]
pub struct GasTariffRecord {
    pub electricity_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_kwh: f64,
}

#[derive(QueryableByName, Debug)]
pub struct StandingChargeRecord {
    #[diesel(sql_type = Timestamp)]
    pub start_date: NaiveDateTime,
    #[diesel(sql_type = Double)]
    pub standing_charge_pence: f64,
}

#[derive(QueryableByName, Debug)]
pub struct UnitPriceRecord {
    #[diesel(sql_type = Timestamp)]
    pub price_effective_time: NaiveDateTime,
    #[diesel(sql_type = Double)]
    pub unit_price_pence: f64,
}

type RepositoryResult<T> = Result<T, RepositoryError>;

pub trait TariffRepository<T, U> {
    fn insert(&self, records: Vec<T>) -> RepositoryResult<()>;

    fn get_standing_charge_history(&self) -> RepositoryResult<Vec<StandingChargeRecord>>;

    fn get_unit_price_history(&self) -> RepositoryResult<Vec<UnitPriceRecord>>;
}

pub struct SqliteElectricityTariffRepository {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl SqliteElectricityTariffRepository {
    pub fn new(conn: Arc<Mutex<SqliteConnection>>) -> Self {
        Self { conn }
    }

    fn get_connection(&self) -> RepositoryResult<MutexGuard<'_, SqliteConnection>> {
        self.conn
            .lock()
            .map_err(|e| RepositoryError::ConnectionError(format!("Could not lock db: {}", e)))
    }
}

impl TariffRepository<ElectricityTariff, ElectricityTariffRecord>
    for SqliteElectricityTariffRepository
{
    fn insert(&self, records: Vec<ElectricityTariff>) -> RepositoryResult<()> {
        let standing_charges = records
            .iter()
            .map(|x| {
                x.standing_charges
                    .iter()
                    .map(|y| NewElectricityStandingCharge {
                        start_date: y.start_date.into(),
                        standing_charge_pence: y.value,
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        self.get_connection()?
            .transaction::<_, diesel::result::Error, _>(|conn| {
                for record in standing_charges {
                    insert_into(electricity_standing_charge::table)
                        .values(&record)
                        .on_conflict(electricity_standing_charge::start_date)
                        .do_update()
                        .set(
                            electricity_standing_charge::standing_charge_pence
                                .eq(excluded(electricity_standing_charge::standing_charge_pence)),
                        ) // Use the correct field reference
                        .execute(conn)?;
                }

                Ok(())
            })?;

        let unit_prices = records
            .iter()
            .map(|x| {
                x.prices.iter().map(|y| NewElectricityUnitPrice {
                    price_effective_time: y.timestamp.into(),
                    unit_price_pence: y.value,
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        self.get_connection()?
            .transaction::<_, diesel::result::Error, _>(|conn| {
                for record in unit_prices {
                    insert_into(electricity_unit_price::table)
                        .values(&record)
                        .on_conflict(electricity_unit_price::price_effective_time)
                        .do_update()
                        .set(
                            electricity_unit_price::unit_price_pence
                                .eq(excluded(electricity_unit_price::unit_price_pence)),
                        )
                        .execute(conn)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    fn get_standing_charge_history(&self) -> RepositoryResult<Vec<StandingChargeRecord>> {
        let mut conn = self.get_connection()?;

        let query = r#"
                WITH price_changes AS (
                    SELECT start_date, standing_charge_pence,
                           LAG(standing_charge_pence) OVER (ORDER BY start_date) AS previous_price
                    FROM electricity_standing_charge
                )
                SELECT pc.start_date, pc.standing_charge_pence
                FROM price_changes AS pc
                WHERE pc.standing_charge_pence <> COALESCE(pc.previous_price, pc.standing_charge_pence)
                   OR pc.previous_price IS NULL
                ORDER BY pc.start_date
            "#;

        Ok(sql_query(query).load::<StandingChargeRecord>(&mut *conn)?)
    }

    fn get_unit_price_history(&self) -> RepositoryResult<Vec<UnitPriceRecord>> {
        let mut conn = self.get_connection()?;

        let query = r#"
                WITH price_changes AS (
                    SELECT price_effective_time, unit_price_pence, LAG(unit_price_pence) OVER (ORDER BY price_effective_time) AS previous_price
                    FROM electricity_unit_price
                )
                SELECT pc.price_effective_time, pc.unit_price_pence
                FROM price_changes AS pc
                WHERE pc.unit_price_pence <> COALESCE(pc.previous_price, pc.unit_price_pence) OR pc.previous_price IS NULL
                ORDER BY pc.price_effective_time
            "#;

        Ok(sql_query(query).load::<UnitPriceRecord>(&mut *conn)?)
    }
}

pub struct SqliteGasTariffRepository {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl SqliteGasTariffRepository {
    pub fn new(conn: Arc<Mutex<SqliteConnection>>) -> Self {
        Self { conn }
    }

    fn get_connection(&self) -> RepositoryResult<MutexGuard<'_, SqliteConnection>> {
        self.conn
            .lock()
            .map_err(|e| RepositoryError::ConnectionError(format!("Could not lock db: {}", e)))
    }
}

impl TariffRepository<GasTariff, GasTariffRecord> for SqliteGasTariffRepository {
    fn insert(&self, records: Vec<GasTariff>) -> RepositoryResult<()> {
        let standing_charges = records
            .iter()
            .map(|x| {
                x.standing_charges
                    .iter()
                    .map(|y| NewGasStandingCharge {
                        start_date: y.start_date.into(),
                        standing_charge_pence: y.value,
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>();

        self.get_connection()?
            .transaction::<_, diesel::result::Error, _>(|conn| {
                for record in standing_charges {
                    insert_into(gas_standing_charge::table)
                        .values(&record)
                        .on_conflict(gas_standing_charge::start_date)
                        .do_update()
                        .set(
                            gas_standing_charge::standing_charge_pence
                                .eq(excluded(gas_standing_charge::standing_charge_pence)),
                        ) // Use the correct field reference
                        .execute(conn)?;
                }

                Ok(())
            })?;

        let unit_prices = records
            .iter()
            .map(|x| {
                x.prices.iter().map(|y| NewGasUnitPrice {
                    price_effective_time: y.timestamp.into(),
                    unit_price_pence: y.value,
                })
            })
            .flatten()
            .collect::<Vec<_>>();

        self.get_connection()?
            .transaction::<_, diesel::result::Error, _>(|conn| {
                for record in unit_prices {
                    insert_into(gas_unit_price::table)
                        .values(&record)
                        .on_conflict(gas_unit_price::price_effective_time)
                        .do_update()
                        .set(
                            gas_unit_price::unit_price_pence
                                .eq(excluded(gas_unit_price::unit_price_pence)),
                        )
                        .execute(conn)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    fn get_standing_charge_history(&self) -> RepositoryResult<Vec<StandingChargeRecord>> {
        let mut conn = self.get_connection()?;

        let query = r#"
                WITH price_changes AS (
                    SELECT start_date, standing_charge_pence,
                           LAG(standing_charge_pence) OVER (ORDER BY start_date) AS previous_price
                    FROM gas_standing_charge
                )
                SELECT pc.start_date, pc.standing_charge_pence
                FROM price_changes AS pc
                WHERE pc.standing_charge_pence <> COALESCE(pc.previous_price, pc.standing_charge_pence)
                   OR pc.previous_price IS NULL
                ORDER BY pc.start_date
            "#;

        Ok(sql_query(query).load::<StandingChargeRecord>(&mut *conn)?)
    }

    fn get_unit_price_history(&self) -> RepositoryResult<Vec<UnitPriceRecord>> {
        let mut conn = self.get_connection()?;

        let query = r#"
                WITH price_changes AS (
                    SELECT price_effective_time, unit_price_pence, LAG(unit_price_pence) OVER (ORDER BY price_effective_time) AS previous_price
                    FROM gas_unit_price
                )
                SELECT pc.price_effective_time, pc.unit_price_pence
                FROM price_changes AS pc
                WHERE pc.unit_price_pence <> COALESCE(pc.previous_price, pc.unit_price_pence) OR pc.previous_price IS NULL
                ORDER BY pc.price_effective_time
            "#;

        Ok(sql_query(query).load::<UnitPriceRecord>(&mut *conn)?)
    }
}
