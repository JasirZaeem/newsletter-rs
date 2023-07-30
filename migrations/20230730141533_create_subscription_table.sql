-- Create table subscription
CREATE TABLE subscription
(
    id            UUID PRIMARY KEY,
    email         TEXT        NOT NULL UNIQUE,
    name          TEXT        NOT NULL,
    subscribed_at timestamptz NOT NULL DEFAULT now()
);
