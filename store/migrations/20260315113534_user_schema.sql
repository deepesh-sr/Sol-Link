-- Add migration script here
CREATE TABLE users(
    id UUID PRIMARY KEY,
    email VARCHAR(30) NOT NULL,
    password TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    public_key TEXT
);

-- when i will need balances from user i will query 
-- SELECT * FROM balances WHERE user_id = '...' 
CREATE TABLE asset (
    id UUID PRIMARY KEY,
    mint_address VARCHAR(32),
    decimals INT,
    name VARCHAR(50),
    symbol VARCHAR(10),
    logo_url VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ 
);

CREATE TABLE balance(
    id UUID PRIMARY KEY ,
    amount BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ, 
    user_id UUID NOT NULL REFERENCES users(id),
    asset_id UUID NOT NULL REFERENCES asset(id),
    UNIQUE(user_id, asset_id)
);

