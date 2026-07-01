CREATE TABLE electricity_consumption_old (
    electricity_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_kwh DOUBLE NOT NULL
);

INSERT INTO electricity_consumption_old
  (electricity_consumption_id, timestamp, energy_consumption_kwh)
SELECT electricity_consumption_id, timestamp, energy_consumption_wh / 1000.0
FROM electricity_consumption;

DROP TABLE electricity_consumption;

ALTER TABLE electricity_consumption_old RENAME TO electricity_consumption;

CREATE TABLE gas_consumption_old (
    gas_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_kwh DOUBLE NOT NULL
);

INSERT INTO gas_consumption_old
  (gas_consumption_id, timestamp, energy_consumption_kwh)
SELECT gas_consumption_id, timestamp, energy_consumption_wh / 1000.0
FROM gas_consumption;

DROP TABLE gas_consumption;

ALTER TABLE gas_consumption_old RENAME TO gas_consumption;
