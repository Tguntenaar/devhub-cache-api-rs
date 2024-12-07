-- Add migration script here
ALTER TABLE last_updated_info ADD COLUMN cursor varchar;

UPDATE last_updated_info SET cursor = '';

-- Add NOT NULL constraint
ALTER TABLE last_updated_info ALTER COLUMN cursor SET NOT NULL;