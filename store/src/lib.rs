use sqlx::{PgPool, postgres::PgPoolOptions};

pub mod user;

#[derive(Debug,Clone)]
pub struct Store { 
    pub pool : PgPool
}

impl Store {
    pub fn new(pool : PgPool)-> Self{
        Store { pool }
    }

    pub async fn connect(url : &str)-> Result<Self, sqlx::Error>{
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(url)
            .await?;

        Ok(Store { pool })
    }

}