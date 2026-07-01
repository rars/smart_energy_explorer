use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{Datelike, NaiveDate, NaiveDateTime};
use chrono_tz::Europe::London;
use diesel::insert_into;
use diesel::SqliteConnection;
use diesel::{prelude::*, upsert::excluded};
use log::error;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::schema::{electricity_consumption, gas_consumption};
use crate::utils::london_midnight_as_utc;

use super::RepositoryError;

const ENERGY_CONSUMPTION_KWH_ERROR_CODE: f64 = 16777.215f64;
const ENERGY_CONSUMPTION_WH_ERROR_CODE: i64 = 16777215i64;

const KWH_TO_WH_SCALE: Decimal = Decimal::ONE_THOUSAND;

pub struct ElectricityConsumptionValue {
    pub timestamp: NaiveDateTime,
    pub value: Decimal,
}

pub struct GasConsumptionValue {
    pub timestamp: NaiveDateTime,
    pub value: Decimal,
}

#[derive(Insertable)]
#[diesel(table_name = electricity_consumption)]
struct NewElectricityConsumption {
    timestamp: NaiveDateTime,
    energy_consumption_wh: i64,
}

#[derive(Insertable)]
#[diesel(table_name = gas_consumption)]
struct NewGasConsumption {
    timestamp: NaiveDateTime,
    energy_consumption_wh: i64,
}

#[derive(Queryable)]
pub struct ElectricityConsumptionRecord {
    pub electricity_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_wh: i64,
}

#[derive(Queryable)]
pub struct GasConsumptionRecord {
    pub gas_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_wh: i64,
}

type RepositoryResult<T> = Result<T, RepositoryError>;

fn group_raw_by_london_day<T, F, G>(
    raw_data_utc: &Vec<T>,
    proj_date_time: F,
    proj_energy: G,
) -> Vec<(NaiveDate, i64)>
where
    F: Fn(&T) -> &NaiveDateTime,
    G: Fn(&T) -> i64,
{
    let mut energy_by_day: BTreeMap<NaiveDate, i64> = BTreeMap::new();

    for elem in raw_data_utc {
        let date_time = proj_date_time(elem);
        let energy = proj_energy(elem);

        let london_date = date_time.and_utc().with_timezone(&London).date_naive();

        *energy_by_day.entry(london_date).or_insert(0) += energy;
    }

    energy_by_day.into_iter().collect()
}

fn group_raw_by_london_month<T, F, G>(
    raw_data_utc: &Vec<T>,
    proj_date_time: F,
    proj_energy: G,
) -> Vec<(NaiveDate, i64)>
where
    F: Fn(&T) -> &NaiveDateTime,
    G: Fn(&T) -> i64,
{
    let mut energy_by_month: BTreeMap<NaiveDate, i64> = BTreeMap::new();

    for elem in raw_data_utc {
        let date_time = proj_date_time(elem);
        let energy = proj_energy(elem);

        let london_first_of_month = date_time
            .and_utc()
            .with_timezone(&London)
            .date_naive()
            .with_day(1)
            .expect("Every month should have 1st of the month");

        *energy_by_month.entry(london_first_of_month).or_insert(0) += energy;
    }

    energy_by_month.into_iter().collect()
}

pub trait ConsumptionRepository<T, U> {
    fn insert(&self, records: Vec<T>) -> RepositoryResult<()>;

    fn get_raw(&self, start: NaiveDate, end: NaiveDate) -> RepositoryResult<Vec<U>>;

    fn get_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>>;

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>>;
}

pub struct SqliteElectricityConsumptionRepository {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl SqliteElectricityConsumptionRepository {
    pub fn new(conn: Arc<Mutex<SqliteConnection>>) -> Self {
        Self { conn }
    }

    fn get_connection(&self) -> RepositoryResult<MutexGuard<'_, SqliteConnection>> {
        self.conn
            .lock()
            .map_err(|_| RepositoryError::SqliteConnectionMutexPoisonedError())
    }
}

impl ConsumptionRepository<ElectricityConsumptionValue, ElectricityConsumptionRecord>
    for SqliteElectricityConsumptionRepository
{
    fn insert(&self, records: Vec<ElectricityConsumptionValue>) -> RepositoryResult<()> {
        let new_records: Vec<_> = records
            .into_iter()
            .map(|x| NewElectricityConsumption {
                timestamp: x.timestamp,
                energy_consumption_wh: (x.value * KWH_TO_WH_SCALE)
                    .to_i64()
                    .expect("Electricity consumption to fit in 64-bit integer"),
            })
            .collect();

        self.get_connection()?
            .transaction::<_, diesel::result::Error, _>(|conn| {
                for record in new_records {
                    insert_into(electricity_consumption::table)
                        .values(&record)
                        .on_conflict(electricity_consumption::timestamp)
                        .do_update()
                        .set(
                            electricity_consumption::energy_consumption_wh
                                .eq(excluded(electricity_consumption::energy_consumption_wh)),
                        ) // Use the correct field reference
                        .execute(conn)?;
                }

                Ok(())
            })?;

        Ok(())
    }

    fn get_raw(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<ElectricityConsumptionRecord>> {
        use crate::schema::electricity_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        Ok(electricity_consumption
            .filter(timestamp.ge(london_midnight_as_utc(&start)))
            .filter(timestamp.lt(london_midnight_as_utc(&end)))
            .load::<ElectricityConsumptionRecord>(&mut *conn)?)
    }

    fn get_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>> {
        let raw_data = self.get_raw(start, end)?;

        Ok(group_raw_by_london_day(
            &raw_data,
            |x| &x.timestamp,
            |x| x.energy_consumption_wh,
        ))
    }

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>> {
        let raw_data = self.get_raw(start, end)?;

        Ok(group_raw_by_london_month(
            &raw_data,
            |x| &x.timestamp,
            |x| x.energy_consumption_wh,
        ))
    }
}

pub struct SqliteGasConsumptionRepository {
    connection: Arc<Mutex<SqliteConnection>>,
}

impl SqliteGasConsumptionRepository {
    pub fn new(connection: Arc<Mutex<SqliteConnection>>) -> Self {
        Self { connection }
    }

    fn get_connection(&self) -> RepositoryResult<MutexGuard<'_, SqliteConnection>> {
        self.connection
            .lock()
            .map_err(|_| RepositoryError::SqliteConnectionMutexPoisonedError())
    }
}

impl ConsumptionRepository<GasConsumptionValue, GasConsumptionRecord>
    for SqliteGasConsumptionRepository
{
    fn insert(&self, records: Vec<GasConsumptionValue>) -> RepositoryResult<()> {
        let new_records: Vec<_> = records
            .into_iter()
            .map(|x| NewGasConsumption {
                timestamp: x.timestamp,
                energy_consumption_wh: (x.value * KWH_TO_WH_SCALE)
                    .to_i64()
                    .expect("Gas consumption to fit in i64"),
            })
            .collect();

        self.get_connection()?
            .transaction::<_, diesel::result::Error, _>(|conn| {
                for record in new_records {
                    insert_into(gas_consumption::table)
                        .values(&record)
                        .on_conflict(gas_consumption::timestamp)
                        .do_update()
                        .set(
                            gas_consumption::energy_consumption_wh
                                .eq(excluded(gas_consumption::energy_consumption_wh)),
                        )
                        .execute(conn)
                        .map_err(|e| {
                            error!("Error inserting new gas consumption entry: {:?}", e);
                            e
                        })?;
                }

                Ok(())
            })?;

        Ok(())
    }

    fn get_raw(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<GasConsumptionRecord>> {
        use crate::schema::gas_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        Ok(gas_consumption
            .filter(timestamp.ge(london_midnight_as_utc(&start)))
            .filter(timestamp.lt(london_midnight_as_utc(&end)))
            .filter(energy_consumption_wh.ne(ENERGY_CONSUMPTION_WH_ERROR_CODE))
            .load::<GasConsumptionRecord>(&mut *conn)?)
    }

    fn get_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>> {
        let raw_data = self.get_raw(start, end)?;

        Ok(group_raw_by_london_day(
            &raw_data,
            |x| &x.timestamp,
            |x| x.energy_consumption_wh,
        ))
    }

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>> {
        let raw_data = self.get_raw(start, end)?;

        Ok(group_raw_by_london_month(
            &raw_data,
            |x| &x.timestamp,
            |x| x.energy_consumption_wh,
        ))
    }
}
