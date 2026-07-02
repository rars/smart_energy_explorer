use chrono::{NaiveDate, NaiveDateTime};
use diesel::dsl::sql;
use diesel::insert_into;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::r2d2::PooledConnection;
use diesel::SqliteConnection;
use diesel::{prelude::*, upsert::excluded};
use log::error;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

use crate::db::SqliteConnectionPool;
use crate::schema::{electricity_consumption, gas_consumption};
use crate::utils::london_date_id_to_naive_date;
use crate::utils::{
    london_midnight_as_utc, naive_date_to_london_date_id, utc_timestamp_to_london_date_id,
};

use super::RepositoryError;

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
    london_date_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = gas_consumption)]
struct NewGasConsumption {
    timestamp: NaiveDateTime,
    energy_consumption_wh: i64,
    london_date_id: i32,
}

#[derive(Queryable)]
pub struct ElectricityConsumptionRecord {
    pub electricity_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_wh: i64,
    pub london_date_id: Option<i32>,
}

#[derive(Queryable)]
pub struct GasConsumptionRecord {
    pub gas_consumption_id: i32,
    pub timestamp: NaiveDateTime,
    pub energy_consumption_wh: i64,
    pub london_date_id: Option<i32>,
}

type RepositoryResult<T> = Result<T, RepositoryError>;

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
    connection_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl SqliteElectricityConsumptionRepository {
    pub fn new(connection_pool: Pool<ConnectionManager<SqliteConnection>>) -> Self {
        Self { connection_pool }
    }

    fn get_connection(
        &self,
    ) -> RepositoryResult<PooledConnection<ConnectionManager<SqliteConnection>>> {
        Ok(self.connection_pool.get()?)
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
                london_date_id: utc_timestamp_to_london_date_id(&x.timestamp),
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
        use crate::schema::electricity_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        let start_london_date_id = naive_date_to_london_date_id(&start);
        let end_london_date_id = naive_date_to_london_date_id(&end);

        let daily_consumption = electricity_consumption
            .filter(london_date_id.is_not_null())
            .filter(london_date_id.ge(start_london_date_id))
            .filter(london_date_id.lt(end_london_date_id))
            .select((
                london_date_id.assume_not_null(),
                sql::<diesel::sql_types::BigInt>("COALESCE(SUM(energy_consumption_wh), 0)"),
            ))
            .group_by(london_date_id)
            .order(london_date_id)
            .load::<(i32, i64)>(&mut *conn)?;

        Ok(daily_consumption
            .iter()
            .map(|(date_id, energy)| {
                let date = london_date_id_to_naive_date(*date_id);
                (date, *energy)
            })
            .collect())
    }

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>> {
        use crate::schema::electricity_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        let start_london_date_id = naive_date_to_london_date_id(&start);
        let end_london_date_id = naive_date_to_london_date_id(&end);

        let london_month_id =
            sql::<diesel::sql_types::Integer>("london_date_id - (london_date_id % 100) + 1");

        let monthly_consumption = electricity_consumption
            .filter(london_date_id.is_not_null())
            .filter(london_date_id.ge(start_london_date_id))
            .filter(london_date_id.lt(end_london_date_id))
            .select((
                london_month_id.clone().assume_not_null(),
                sql::<diesel::sql_types::BigInt>("COALESCE(SUM(energy_consumption_wh), 0)"),
            ))
            .group_by(london_month_id.clone())
            .order(london_month_id)
            .load::<(i32, i64)>(&mut *conn)?;

        Ok(monthly_consumption
            .iter()
            .map(|(date_id, energy)| {
                let date = london_date_id_to_naive_date(*date_id);
                (date, *energy)
            })
            .collect())
    }
}

pub struct SqliteGasConsumptionRepository {
    connection_pool: SqliteConnectionPool,
}

impl SqliteGasConsumptionRepository {
    pub fn new(connection_pool: SqliteConnectionPool) -> Self {
        Self { connection_pool }
    }

    fn get_connection(
        &self,
    ) -> RepositoryResult<PooledConnection<ConnectionManager<SqliteConnection>>> {
        Ok(self.connection_pool.get()?)
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
                london_date_id: utc_timestamp_to_london_date_id(&x.timestamp),
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
        use crate::schema::gas_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        let start_london_date_id = naive_date_to_london_date_id(&start);
        let end_london_date_id = naive_date_to_london_date_id(&end);

        let daily_consumption = gas_consumption
            .filter(london_date_id.is_not_null())
            .filter(london_date_id.ge(start_london_date_id))
            .filter(london_date_id.lt(end_london_date_id))
            .select((
                london_date_id.assume_not_null(),
                sql::<diesel::sql_types::BigInt>("COALESCE(SUM(energy_consumption_wh), 0)"),
            ))
            .group_by(london_date_id)
            .order(london_date_id)
            .load::<(i32, i64)>(&mut *conn)?;

        Ok(daily_consumption
            .iter()
            .map(|(date_id, energy)| {
                let date = london_date_id_to_naive_date(*date_id);
                (date, *energy)
            })
            .collect())
    }

    fn get_monthly(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> RepositoryResult<Vec<(NaiveDate, i64)>> {
        use crate::schema::gas_consumption::dsl::*;

        let mut conn = self.get_connection()?;

        let start_london_date_id = naive_date_to_london_date_id(&start);
        let end_london_date_id = naive_date_to_london_date_id(&end);

        let london_month_id =
            sql::<diesel::sql_types::Integer>("london_date_id - (london_date_id % 100) + 1");

        let monthly_consumption = gas_consumption
            .filter(london_date_id.is_not_null())
            .filter(london_date_id.ge(start_london_date_id))
            .filter(london_date_id.lt(end_london_date_id))
            .select((
                london_month_id.clone().assume_not_null(),
                sql::<diesel::sql_types::BigInt>("COALESCE(SUM(energy_consumption_wh), 0)"),
            ))
            .group_by(london_month_id.clone())
            .order(london_month_id)
            .load::<(i32, i64)>(&mut *conn)?;

        Ok(monthly_consumption
            .iter()
            .map(|(date_id, energy)| {
                let date = london_date_id_to_naive_date(*date_id);
                (date, *energy)
            })
            .collect())
    }
}
