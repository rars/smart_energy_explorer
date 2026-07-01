CREATE TABLE electricity_consumption_new (
    electricity_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_wh BIGINT NOT NULL
);

INSERT INTO electricity_consumption_new
  (electricity_consumption_id, timestamp, energy_consumption_wh)
SELECT electricity_consumption_id, timestamp, CAST(ROUND(energy_consumption_kwh * 1000) AS BIGINT)
FROM electricity_consumption;

DROP TABLE electricity_consumption;

ALTER TABLE electricity_consumption_new RENAME TO electricity_consumption;

CREATE TABLE gas_consumption_new (
    gas_consumption_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    timestamp DATETIME NOT NULL UNIQUE,
    energy_consumption_wh BIGINT NOT NULL
);

INSERT INTO gas_consumption_new
  (gas_consumption_id, timestamp, energy_consumption_wh)
SELECT gas_consumption_id, timestamp, CAST(ROUND(energy_consumption_kwh * 1000) AS BIGINT)
FROM gas_consumption;

DROP TABLE gas_consumption;

ALTER TABLE gas_consumption_new RENAME TO gas_consumption;
