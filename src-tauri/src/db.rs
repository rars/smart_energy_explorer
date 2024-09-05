use crate::data::energy_profile::EnergyProfile;
use crate::schema::{electricity_consumption, energy_profile, gas_consumption};
use chrono::{Local, NaiveDate, NaiveDateTime};
use diesel::dsl::sql;
use diesel::insert_into;
use diesel::sql_types::{Date, Double};
use diesel::sqlite::SqliteConnection;
use diesel::{prelude::*, upsert::excluded};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::error;
use n3rgy::{ElectricityConsumption, GasConsumption};
use serde::Serialize;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(conn: &mut SqliteConnection) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");
}

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    // dotenvy::dotenv().ok();
    // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
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
    energy_consumption_m3: f64,
}
/*
#[derive(Serialize, Queryable, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnergyProfile {
    pub energy_profile_id: i32,
    pub name: String,
    pub is_active: bool,
    pub start_date: NaiveDateTime,
    pub last_date_retrieved: Option<NaiveDateTime>,
}
    */

#[derive(Insertable)]
#[diesel(table_name = energy_profile)]
struct NewEnergyProfile<'a> {
    name: &'a str,
    is_active: bool,
    start_date: NaiveDateTime,
}

pub fn get_energy_profile(conn: &mut SqliteConnection, name: &str) -> QueryResult<EnergyProfile> {
    energy_profile::table
        .filter(energy_profile::name.eq(name))
        .get_result(conn)
}

pub fn get_all_energy_profiles(conn: &mut SqliteConnection) -> QueryResult<Vec<EnergyProfile>> {
    energy_profile::table.load::<EnergyProfile>(conn)
}

pub fn create_energy_profile<T: AsRef<str>>(
    conn: &mut SqliteConnection,
    name: T,
) -> QueryResult<EnergyProfile> {
    let name_ref = name.as_ref();
    let start_date = Local::now().naive_local();

    let new_profile = NewEnergyProfile {
        name: name_ref,
        is_active: true,
        start_date,
    };

    diesel::insert_into(energy_profile::table)
        .values(&new_profile)
        .execute(conn)?;

    get_energy_profile(conn, name_ref)
}

pub fn update_energy_profile(
    conn: &mut SqliteConnection,
    energy_profile_id_param: i32,
    new_is_active: bool,
    new_start_date: NaiveDateTime,
    new_last_date_retrieved: NaiveDateTime,
) -> QueryResult<EnergyProfile> {
    use crate::schema::energy_profile::dsl::*;

    diesel::update(energy_profile.find(energy_profile_id_param))
        .set((
            is_active.eq(new_is_active),
            start_date.eq(new_start_date),
            last_date_retrieved.eq(new_last_date_retrieved),
        ))
        .execute(conn)?;

    energy_profile.find(energy_profile_id_param).first(conn)
}

pub fn update_energy_profile_settings(
    conn: &mut SqliteConnection,
    energy_profile_id_param: i32,
    new_is_active: bool,
    new_start_date: NaiveDateTime,
) -> QueryResult<EnergyProfile> {
    use crate::schema::energy_profile::dsl::*;

    diesel::update(energy_profile.find(energy_profile_id_param))
        .set((is_active.eq(new_is_active), start_date.eq(new_start_date)))
        .execute(conn)?;

    energy_profile.find(energy_profile_id_param).first(conn)
}

pub fn insert_electricity_consumption(
    conn: &mut SqliteConnection,
    records: Vec<ElectricityConsumption>,
) {
    let new_records: Vec<_> = records
        .into_iter()
        .map(|x| NewElectricityConsumption {
            timestamp: x.timestamp,
            energy_consumption_kwh: x.value,
        })
        .collect();

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        for record in new_records {
            insert_into(electricity_consumption::table)
                .values(&record)
                .on_conflict(electricity_consumption::timestamp)
                .do_update()
                .set(
                    electricity_consumption::energy_consumption_kwh
                        .eq(excluded(electricity_consumption::energy_consumption_kwh)),
                ) // Use the correct field reference
                .execute(conn)
                .expect("Error inserting new electricity consumption entry");
        }

        Ok(())
    })
    .expect("Error during transaction to insert new records");
}

pub fn insert_gas_consumption(conn: &mut SqliteConnection, records: Vec<GasConsumption>) {
    let new_records: Vec<_> = records
        .into_iter()
        .map(|x| NewGasConsumption {
            timestamp: x.timestamp,
            energy_consumption_m3: x.value,
        })
        .collect();

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        for record in new_records {
            insert_into(gas_consumption::table)
                .values(&record)
                .on_conflict(gas_consumption::timestamp)
                .do_update()
                .set(
                    gas_consumption::energy_consumption_m3
                        .eq(excluded(gas_consumption::energy_consumption_m3)),
                )
                .execute(conn)
                .map_err(|e| {
                    error!("Error inserting new gas consumption entry: {:?}", e);
                    e
                })?;
        }

        Ok(())
    })
    .expect("Error during transaction to insert new records");
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
    pub energy_consumption_m3: f64,
}

pub fn get_raw_electricity_consumption(
    connection: &mut SqliteConnection,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<ElectricityConsumptionRecord>> {
    use crate::schema::electricity_consumption::dsl::*;

    electricity_consumption
        .filter(timestamp.ge(NaiveDateTime::from(start)))
        .filter(timestamp.lt(NaiveDateTime::from(end)))
        .load::<ElectricityConsumptionRecord>(connection)
}

pub fn get_raw_gas_consumption(
    connection: &mut SqliteConnection,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<GasConsumptionRecord>> {
    use crate::schema::gas_consumption::dsl::*;

    gas_consumption
        .filter(timestamp.ge(NaiveDateTime::from(start)))
        .filter(timestamp.lt(NaiveDateTime::from(end)))
        .load::<GasConsumptionRecord>(connection)
}

pub fn get_daily_electricity_consumption(
    connection: &mut SqliteConnection,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<(NaiveDate, f64)>> {
    use crate::schema::electricity_consumption::dsl::*;

    electricity_consumption
        .filter(timestamp.ge(start.and_hms_opt(0, 0, 0).unwrap()))
        .filter(timestamp.lt(end.and_hms_opt(0, 0, 0).unwrap()))
        .select((
            sql::<Date>("DATE(timestamp)"),
            sql::<Double>("COALESCE(SUM(energy_consumption_kwh), 0.0)"),
        ))
        .group_by(sql::<Date>("DATE(timestamp)"))
        .order(sql::<Date>("DATE(timestamp)"))
        .load::<(NaiveDate, f64)>(connection)
}

pub fn get_daily_gas_consumption(
    connection: &mut SqliteConnection,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<(NaiveDate, f64)>> {
    use crate::schema::gas_consumption::dsl::*;

    gas_consumption
        .filter(timestamp.ge(start.and_hms_opt(0, 0, 0).unwrap()))
        .filter(timestamp.lt(end.and_hms_opt(0, 0, 0).unwrap()))
        .select((
            sql::<Date>("DATE(timestamp)"),
            sql::<Double>("COALESCE(SUM(energy_consumption_m3), 0.0)"),
        ))
        .group_by(sql::<Date>("DATE(timestamp)"))
        .order(sql::<Date>("DATE(timestamp)"))
        .load::<(NaiveDate, f64)>(connection)
}

pub fn get_monthly_electricity_consumption(
    connection: &mut SqliteConnection,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<(NaiveDate, f64)>> {
    use crate::schema::electricity_consumption::dsl::*;

    electricity_consumption
        .filter(timestamp.ge(start.and_hms_opt(0, 0, 0).unwrap()))
        .filter(timestamp.lt(end.and_hms_opt(0, 0, 0).unwrap()))
        .select((
            sql::<Date>("DATE(timestamp, 'start of month')"),
            sql::<Double>("COALESCE(SUM(energy_consumption_kwh), 0.0)"),
        ))
        .group_by(sql::<Date>("DATE(timestamp, 'start of month')"))
        .order(sql::<Date>("DATE(timestamp, 'start of month')"))
        .load::<(NaiveDate, f64)>(connection)
}

pub fn get_monthly_gas_consumption(
    connection: &mut SqliteConnection,
    start: NaiveDate,
    end: NaiveDate,
) -> QueryResult<Vec<(NaiveDate, f64)>> {
    use crate::schema::gas_consumption::dsl::*;

    gas_consumption
        .filter(timestamp.ge(start.and_hms_opt(0, 0, 0).unwrap()))
        .filter(timestamp.lt(end.and_hms_opt(0, 0, 0).unwrap()))
        .select((
            sql::<Date>("DATE(timestamp, 'start of month')"),
            sql::<Double>("COALESCE(SUM(energy_consumption_m3), 0.0)"),
        ))
        .group_by(sql::<Date>("DATE(timestamp, 'start of month')"))
        .order(sql::<Date>("DATE(timestamp, 'start of month')"))
        .load::<(NaiveDate, f64)>(connection)
}
