-- Rename the table
ALTER TABLE after_date RENAME TO last_updated_info;

-- Add new columns
ALTER TABLE last_updated_info ADD COLUMN id SERIAL PRIMARY KEY;
ALTER TABLE last_updated_info ADD COLUMN after_block BIGINT;  -- Remove NOT NULL initially

-- Migrate existing data
UPDATE last_updated_info SET after_block = 0;  -- Set default value for all rows

-- Now make the column NOT NULL
ALTER TABLE last_updated_info ALTER COLUMN after_block SET NOT NULL;

-- Insert initial record if table is empty
INSERT INTO last_updated_info (after_date, after_block)
SELECT 0, 0
WHERE NOT EXISTS (SELECT 1 FROM last_updated_info);