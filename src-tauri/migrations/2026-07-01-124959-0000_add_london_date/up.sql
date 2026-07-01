
ALTER TABLE electricity_consumption ADD COLUMN london_date_id INTEGER;
ALTER TABLE gas_consumption ADD COLUMN london_date_id INTEGER;

CREATE INDEX idx_electricity_consumption_london_date_id ON electricity_consumption(london_date_id);
CREATE INDEX idx_gas_consumption_london_date_id ON gas_consumption(london_date_id);
