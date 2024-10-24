-- Add migration script here
CREATE TABLE IF NOT EXISTS after_date (
    after_date BIGINT NOT NULL
);

INSERT INTO after_date (after_date) VALUES (0);
