CREATE TABLE IF NOT EXISTS electricity_consumption (
    electricity_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_kwh DOUBLE NOT NULL
);

CREATE TABLE IF NOT EXISTS gas_consumption (
    gas_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_m3 DOUBLE NOT NULL
);
