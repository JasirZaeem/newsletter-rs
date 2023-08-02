-- Create subscription token table
CREATE TABLE subscription_token (
    token uuid NOT NULL,
    subscriber_id uuid NOT NULL REFERENCES subscription (id),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (token)
);