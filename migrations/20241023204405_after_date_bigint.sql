-- Add migration script here

ALTER TABLE after_date
DROP COLUMN after_date;

ALTER TABLE after_date
ADD COLUMN after_date BIGINT NOT NULL;
