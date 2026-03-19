-- Add migration script here
CREATE TABLE keyshares(
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL,
  key_package BYTEA NOT NULL,      -- encrypted keyshare
  public_key_package BYTEA NOT NULL, -- group public key info
  created_at TIMESTAMPTZ DEFAULT NOW()
);