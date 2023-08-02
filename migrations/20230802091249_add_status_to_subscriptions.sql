-- Add confirmed_at column to subscription table
-- A timestamp will be added to this column when a user clicks the confirmation link
ALTER TABLE subscription
    ADD COLUMN confirmed_at timestamptz DEFAULT NULL;