-- Add migration script here
ALTER TABLE asset ADD CONSTRAINT asset_mint_address_unique UNIQUE (mint_address);
