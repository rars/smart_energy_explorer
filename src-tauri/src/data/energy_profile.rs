use chrono::{Local, NaiveDateTime};
// use diesel::{QueryResult, Queryable, SqliteConnection};
use diesel::{prelude::*, upsert::excluded};
use serde::Serialize;

use crate::schema::energy_profile;

#[derive(Serialize, Queryable, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EnergyProfile {
    pub energy_profile_id: i32,
    pub name: String,
    pub is_active: bool,
    pub start_date: NaiveDateTime,
    pub last_date_retrieved: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = energy_profile)]
struct NewEnergyProfile<'a> {
    name: &'a str,
    is_active: bool,
    start_date: NaiveDateTime,
}

pub trait EnergyProfileRepository {
    fn get_energy_profile(&mut self, name: &str) -> QueryResult<EnergyProfile>;
    fn create_energy_profile(&mut self, name: &str) -> QueryResult<EnergyProfile>;
}

pub struct SqliteEnergyProfileRepository<'a> {
    conn: &'a mut SqliteConnection,
}

impl SqliteEnergyProfileRepository<'_> {
    pub fn new(conn: &mut SqliteConnection) -> SqliteEnergyProfileRepository {
        SqliteEnergyProfileRepository { conn }
    }

    fn get_connection(&mut self) -> &mut SqliteConnection {
        self.conn
    }
}

impl EnergyProfileRepository for SqliteEnergyProfileRepository<'_> {
    fn get_energy_profile(&mut self, name: &str) -> QueryResult<EnergyProfile> {
        energy_profile::table
            .filter(energy_profile::name.eq(name))
            .get_result(self.get_connection())
    }

    fn create_energy_profile(&mut self, name: &str) -> QueryResult<EnergyProfile> {
        let start_date = Local::now().naive_local();

        let new_profile = NewEnergyProfile {
            name: name,
            is_active: true,
            start_date,
        };

        diesel::insert_into(energy_profile::table)
            .values(&new_profile)
            .execute(self.get_connection())?;

        self.get_energy_profile(name)
    }
}
/*
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
*/
