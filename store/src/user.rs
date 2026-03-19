use core::str;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::Store;

#[derive(Serialize, Debug, FromRow)]
pub struct User {
    id: Uuid,
    email: String,
    #[serde(skip_serializing)]
    password: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    public_key: Option<String>,
}

#[derive(Serialize, Debug, FromRow)]
pub struct Asset {
    id: Uuid,
    mint_address: String,
    decimals: i32,
    name: String,
    symbol: String,
    logo_url: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Debug, FromRow)]
pub struct Balance {
    id: Uuid,
    amount: i64,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    user_id: Uuid,
    asset_id: Uuid,
}

#[derive(Deserialize, Serialize, FromRow, Debug)]
pub struct UserBalance{
    pub amount: i64,
    pub mint_address: String,
    pub decimals: i32,
    pub name: String,
    pub symbol: String,
    pub logo_url: Option<String>,
}

impl Store {
    pub async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        public_key: Option<&str>,
    ) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users(email, password, public_key)
            VALUES ( $1,$2,$3)
            RETURNING *
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(public_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT * FROM users WHERE email=($1)
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;
        Ok(user)
    }
    pub async fn get_user_by_id(&self, id: &Uuid) -> Result<User, sqlx::Error> {
        let user = sqlx::query_as(r#"SELECT * FROM users WHERE id = $1"#)
            .bind(&id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn upsert_asset(
        &self,
        mint_address: &str,
        decimals: i32,
        name: &str,
        symbol: &str,
        logo_url: Option<&str>,
    ) -> Result<Asset, sqlx::Error> {

        let asset = sqlx::query_as::<_,Asset>(
            r#"
            INSERT INTO asset (mint_address, decimals, name, symbol, logo_url)
            VALUES ($1,$2,$3,$4,$5)
            ON CONFLICT (mint_address) DO UPDATE 
                SET name = EXCLUDED.name,
                symbol = EXCLUDED.symbol,
                logo_url = EXCLUDED.logo_url,
                updated_at = NOW()
            RETURNING *
            "#
        ).bind(mint_address)
        .bind(decimals)
        .bind(name)
        .bind(symbol)
        .bind(logo_url)
        .fetch_one(&self.pool)
        .await?;
    Ok(asset)
    }

    pub async fn upsert_balance(
        &self,
        amount : i64, 
        user_id : Uuid,
        asset_id : Uuid
    ) -> Result<Balance, sqlx::Error> {

        let balance = sqlx::query_as::<_,Balance>(
            r#"
            INSERT INTO balance (amount, user_id, asset_id)
            VALUES ($1,$2,$3)
            ON CONFLICT (user_id, asset_id) DO UPDATE 
                SET amount = EXCLUDED.amount,
                    updated_at = NOW()
            RETURNING *
            "#
        ).bind(amount)
        .bind(user_id)
        .bind(asset_id)
        .fetch_one(&self.pool)
        .await?;
    Ok(balance)

    }

    pub async fn get_asset_by_mint(&self, mint_address : &str) -> Result<Asset , sqlx::Error>{
        let asset =sqlx::query_as::<_,Asset>(
            r#"
            SELECT * FROM asset 
            WHERE mint_address = $1
            "#
        ).bind(mint_address)
        .fetch_one(&self.pool)
        .await?;

        Ok(asset)
    }

    pub async fn get_balance_by_user(&self,user_id : Uuid)->Result<Vec<UserBalance>, sqlx::Error>{
        
        let user_balance = sqlx::query_as::<_,UserBalance>(
            r#"
            SELECT  b.amount, a.mint_address, a.decimals, a.name, a.symbol, a.logo_url
            FROM balance b
            JOIN asset a ON b.asset_id = a.id
            WHERE b.user_id = $1
            "#
        ).bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(user_balance)
    }
}

impl User {
    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_pubkey(&self) -> Option<&str> {
        self.public_key.as_deref()
    }
}
