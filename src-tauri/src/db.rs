use chrono_tz::Europe::London;
use diesel::prelude::*;
use diesel::{sqlite::SqliteConnection, Connection, ExpressionMethods};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::info;

use crate::data::RepositoryError;
use crate::utils::utc_timestamp_to_london_date_id;
use crate::{
    schema::electricity_consumption::{london_date_id, table},
    AppError,
};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(conn: &mut SqliteConnection) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");
}

pub fn revert_all_migrations(conn: &mut SqliteConnection) {
    conn.revert_all_migrations(MIGRATIONS)
        .expect("Error reverting migrations");
}

pub fn establish_connection(database_url: &str) -> SqliteConnection {
    // dotenvy::dotenv().ok();
    // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn populate_missing_london_date_ids(
    conn: &mut SqliteConnection,
) -> Result<(), RepositoryError> {
    const BATCH_SIZE: i64 = 5000;

    {
        use crate::schema::electricity_consumption::dsl::*;

        loop {
            let records = electricity_consumption
                .filter(london_date_id.is_null())
                .limit(BATCH_SIZE)
                .load::<crate::data::consumption::ElectricityConsumptionRecord>(conn)?;

            if records.is_empty() {
                break;
            }

            info!(
                "Populating london_date_id for {} electricity consumptionrecords",
                records.len()
            );

            conn.transaction::<_, diesel::result::Error, _>(|transaction_conn| {
                for row in records {
                    let date_id = utc_timestamp_to_london_date_id(&row.timestamp);

                    diesel::update(electricity_consumption.find(row.electricity_consumption_id))
                        .set(london_date_id.eq(date_id))
                        .execute(transaction_conn)?;
                }

                Ok(())
            })?;
        }
    }

    {
        use crate::schema::gas_consumption::dsl::*;

        loop {
            let records = gas_consumption
                .filter(london_date_id.is_null())
                .limit(BATCH_SIZE)
                .load::<crate::data::consumption::GasConsumptionRecord>(conn)?;

            if records.is_empty() {
                break;
            }

            info!(
                "Populating london_date_id for {} gas consumption records",
                records.len()
            );

            conn.transaction::<_, diesel::result::Error, _>(|transaction_conn| {
                for row in records {
                    let date_id = utc_timestamp_to_london_date_id(&row.timestamp);

                    diesel::update(gas_consumption.find(row.gas_consumption_id))
                        .set(london_date_id.eq(date_id))
                        .execute(transaction_conn)?;
                }

                Ok(())
            })?;
        }
    }

    Ok(())
}
