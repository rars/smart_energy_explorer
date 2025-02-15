CREATE TABLE IF NOT EXISTS electricity_standing_charge (
    electricity_standing_charge_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    start_date DATETIME NOT NULL UNIQUE,
    standing_charge_pence DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS gas_standing_charge (
    gas_standing_charge_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    start_date DATETIME NOT NULL UNIQUE,
    standing_charge_pence DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS electricity_unit_price (
    electricity_unit_price_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    price_effective_time DATETIME NOT NULL UNIQUE,
    unit_price_pence DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS gas_unit_price (
    gas_unit_price_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    price_effective_time DATETIME NOT NULL UNIQUE,
    unit_price_pence DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS electricity_tariff_plan (
    tariff_id TEXT NOT NULL PRIMARY KEY,
    plan TEXT NOT NULL,
    effective_date DATETIME NOT NULL UNIQUE,
    display_name TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS gas_tariff_plan (
    tariff_id TEXT NOT NULL PRIMARY KEY,
    plan TEXT NOT NULL,
    effective_date DATETIME NOT NULL UNIQUE,
    display_name TEXT NOT NULL
);
