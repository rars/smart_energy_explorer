CREATE TABLE IF NOT EXISTS energy_profile (
    energy_profile_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(100) NOT NULL UNIQUE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    start_date DATETIME NOT NULL,
    last_date_retrieved DATETIME NULL
);
