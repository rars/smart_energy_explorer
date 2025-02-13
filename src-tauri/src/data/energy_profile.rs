use std::sync::{Arc, Mutex, MutexGuard};

use chrono::{Datelike, Local, NaiveDateTime};
use diesel::dsl::*;
use diesel::prelude::*;
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
    pub base_unit: String,
}

#[derive(Insertable)]
#[diesel(table_name = energy_profile)]
struct NewEnergyProfile<'a> {
    name: &'a str,
    is_active: bool,
    start_date: NaiveDateTime,
    base_unit: &'a str,
}

pub trait EnergyProfileRepository {
    fn get_energy_profile(&self, name: &str) -> QueryResult<EnergyProfile>;
    fn get_all_energy_profiles(&self) -> QueryResult<Vec<EnergyProfile>>;
    fn create_energy_profile(&self, name: &str, base_unit: &str) -> QueryResult<EnergyProfile>;
    fn update_energy_profile(
        &self,
        energy_profile_id_param: i32,
        new_is_active: bool,
        new_start_date: NaiveDateTime,
        new_last_date_retrieved: NaiveDateTime,
    ) -> QueryResult<EnergyProfile>;
    fn update_energy_profile_settings(
        &self,
        energy_profile_id_param: i32,
        new_is_active: bool,
        new_start_date: NaiveDateTime,
    ) -> QueryResult<EnergyProfile>;
}

pub struct SqliteEnergyProfileRepository {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl SqliteEnergyProfileRepository {
    pub fn new(conn: Arc<Mutex<SqliteConnection>>) -> Self {
        Self { conn }
    }

    fn connection(&self) -> MutexGuard<'_, SqliteConnection> {
        self.conn
            .lock()
            .expect("Could not acquire lock on SqliteConnection")
    }
}

impl EnergyProfileRepository for SqliteEnergyProfileRepository {
    fn get_energy_profile(&self, name: &str) -> QueryResult<EnergyProfile> {
        let mut conn = self.connection();

        energy_profile::table
            .filter(energy_profile::name.eq(name))
            .get_result(&mut *conn)
    }

    fn get_all_energy_profiles(&self) -> QueryResult<Vec<EnergyProfile>> {
        let mut conn = self.connection();
        energy_profile::table.load::<EnergyProfile>(&mut *conn)
    }

    fn create_energy_profile(&self, name: &str, base_unit: &str) -> QueryResult<EnergyProfile> {
        let start_of_current_month = Local::now().date_naive().with_day(1).unwrap();
        let end_of_previous_month = start_of_current_month.pred_opt().unwrap();

        let start_date: NaiveDateTime = end_of_previous_month.with_day(1).unwrap().into();

        let new_profile = NewEnergyProfile {
            name: name,
            is_active: true,
            start_date,
            base_unit,
        };

        let mut conn = self.connection();

        diesel::insert_into(energy_profile::table)
            .values(&new_profile)
            .execute(&mut *conn)?;

        energy_profile::table
            .filter(energy_profile::name.eq(name))
            .get_result(&mut *conn)
    }

    fn update_energy_profile(
        &self,
        energy_profile_id_param: i32,
        new_is_active: bool,
        new_start_date: NaiveDateTime,
        new_last_date_retrieved: NaiveDateTime,
    ) -> QueryResult<EnergyProfile> {
        use crate::schema::energy_profile::dsl::*;

        let mut conn = self.connection();

        diesel::update(energy_profile.find(energy_profile_id_param))
            .set((
                is_active.eq(new_is_active),
                start_date.eq(new_start_date),
                last_date_retrieved.eq(case_when(
                    start_date.gt(new_start_date),
                    None::<NaiveDateTime>,
                )
                .otherwise(new_last_date_retrieved)),
            ))
            .execute(&mut *conn)?;

        energy_profile
            .find(energy_profile_id_param)
            .first(&mut *conn)
    }

    fn update_energy_profile_settings(
        &self,
        energy_profile_id_param: i32,
        new_is_active: bool,
        new_start_date: NaiveDateTime,
    ) -> QueryResult<EnergyProfile> {
        use crate::schema::energy_profile::dsl::*;

        let mut conn = self.connection();

        diesel::update(energy_profile.find(energy_profile_id_param))
            .set((
                is_active.eq(new_is_active),
                start_date.eq(new_start_date),
                last_date_retrieved.eq(case_when(
                    start_date.gt(new_start_date),
                    None::<NaiveDateTime>,
                )
                .otherwise(last_date_retrieved)),
            ))
            .execute(&mut *conn)?;

        energy_profile
            .find(energy_profile_id_param)
            .first(&mut *conn)
    }
}
