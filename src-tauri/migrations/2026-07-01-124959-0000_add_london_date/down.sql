-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_electricity_consumption_london_date_id;
DROP INDEX IF EXISTS idx_gas_consumption_london_date_id;

ALTER TABLE electricity_consumption DROP COLUMN london_date_id;
ALTER TABLE gas_consumption DROP COLUMN london_date_id;
