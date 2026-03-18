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
}

impl User {
    pub fn get_password(&self) -> &str {
        &self.password
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn get_pubkey(&self)-> Option<&str>{
        self.public_key.as_deref()
    }
}
