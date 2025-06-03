use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{NaiveDate, NaiveDateTime};
use diesel::dsl::sql;
use diesel::insert_into;
use diesel::sql_types::{Date, Double};
use diesel::SqliteConnection;
use diesel::{prelude::*, upsert::excluded};
use log::error;

use crate::schema::{electricity_consumption, gas_consumption};

use super::RepositoryError;

const ENERGY_CONSUMPTION_KWH_ERROR_CODE: f64 = 16777.215f64;

pub struct ElectricityConsumptionValue {
    pub timestamp: NaiveDateTime,
    pub value: f64,
}

pub struct GasConsumptionValue {
    pub timestamp: NaiveDateTime,
    pub value: f64,
}

#[derive(Insertable)]
#[diesel(table_name = electricity_consumption)]
struct NewElectricityConsumption {
    timestamp: NaiveDateTime,
    energy_consumption_kwh: f64,
}

#[derive(Insertable)]
#[diesel(table_name = gas_consumption)]
struct NewGasConsumption {
    timestamp: NaiveDateTime,
    energy_consumption_kwh: f64,
}

#[derive(Queryable)]
pub struct ElectricityConsumptionRecord {
    pub electricity_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_kwh: f64,
}

#[derive(Queryable)]
pub struct GasConsumptionRecord {
    pub gas_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_kwh: f64,
}

type RepositoryResult<T> = Result<T, RepositoryError>;

pub trait ConsumptionRepository<T, U> {
    fn insert(&self, records: Vec<T>) -> RepositoryResult<()>;

    fn get_raw(&self, start: NaiveDate, end: NaiveDate) -> RepositoryResult<Vec<U>>;

    fn get_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, f64)>>;

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, f64)>>;
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
                energy_consumption_kwh: x.value,
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
                            electricity_consumption::energy_consumption_kwh
                                .eq(excluded(electricity_consumption::energy_consumption_kwh)),
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
            .filter(timestamp.ge(NaiveDateTime::from(start)))
            .filter(timestamp.lt(NaiveDateTime::from(end)))
            .load::<ElectricityConsumptionRecord>(&mut *conn)?)
    }

    fn get_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, f64)>> {
        use crate::schema::electricity_consumption::dsl::*;

        let mut connection = self.get_connection()?;

        Ok(electricity_consumption
            .filter(timestamp.ge(NaiveDateTime::from(start)))
            .filter(timestamp.lt(NaiveDateTime::from(end)))
            .select((
                sql::<Date>("DATE(timestamp)"),
                sql::<Double>("COALESCE(SUM(energy_consumption_kwh), 0.0)"),
            ))
            .group_by(sql::<Date>("DATE(timestamp)"))
            .order(sql::<Date>("DATE(timestamp)"))
            .load::<(NaiveDate, f64)>(&mut *connection)?)
    }

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, f64)>> {
        use crate::schema::electricity_consumption::dsl::*;

        let mut connection = self.get_connection()?;

        Ok(electricity_consumption
            .filter(timestamp.ge(NaiveDateTime::from(start)))
            .filter(timestamp.lt(NaiveDateTime::from(end)))
            .select((
                sql::<Date>("DATE(timestamp, 'start of month')"),
                sql::<Double>("COALESCE(SUM(energy_consumption_kwh), 0.0)"),
            ))
            .group_by(sql::<Date>("DATE(timestamp, 'start of month')"))
            .order(sql::<Date>("DATE(timestamp, 'start of month')"))
            .load::<(NaiveDate, f64)>(&mut *connection)?)
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
                energy_consumption_kwh: x.value,
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
                            gas_consumption::energy_consumption_kwh
                                .eq(excluded(gas_consumption::energy_consumption_kwh)),
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
            .filter(timestamp.ge(NaiveDateTime::from(start)))
            .filter(timestamp.lt(NaiveDateTime::from(end)))
            .filter(energy_consumption_kwh.ne(ENERGY_CONSUMPTION_KWH_ERROR_CODE))
            .load::<GasConsumptionRecord>(&mut *conn)?)
    }

    fn get_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, f64)>> {
        use crate::schema::gas_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        Ok(gas_consumption
            .filter(timestamp.ge(NaiveDateTime::from(start)))
            .filter(timestamp.lt(NaiveDateTime::from(end)))
            .filter(energy_consumption_kwh.ne(ENERGY_CONSUMPTION_KWH_ERROR_CODE))
            .select((
                sql::<Date>("DATE(timestamp)"),
                sql::<Double>("COALESCE(SUM(energy_consumption_kwh), 0.0)"),
            ))
            .group_by(sql::<Date>("DATE(timestamp)"))
            .order(sql::<Date>("DATE(timestamp)"))
            .load::<(NaiveDate, f64)>(&mut *conn)?)
    }

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, f64)>> {
        use crate::schema::gas_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        Ok(gas_consumption
            .filter(timestamp.ge(NaiveDateTime::from(start)))
            .filter(timestamp.lt(NaiveDateTime::from(end)))
            .filter(energy_consumption_kwh.ne(ENERGY_CONSUMPTION_KWH_ERROR_CODE))
            .select((
                sql::<Date>("DATE(timestamp, 'start of month')"),
                sql::<Double>("COALESCE(SUM(energy_consumption_kwh), 0.0)"),
            ))
            .group_by(sql::<Date>("DATE(timestamp, 'start of month')"))
            .order(sql::<Date>("DATE(timestamp, 'start of month')"))
            .load::<(NaiveDate, f64)>(&mut *conn)?)
    }
}
