-- Add migration script here
ALTER TABLE Pictures
ADD COLUMN json TEXT NOT NULL;
