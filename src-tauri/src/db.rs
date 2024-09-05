use diesel::{sqlite::SqliteConnection, Connection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

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
