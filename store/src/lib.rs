use sqlx::PgPool;

pub mod user;

pub struct Store { 
    pool : PgPool
}