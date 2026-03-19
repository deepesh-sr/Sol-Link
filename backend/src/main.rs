mod routes;
 
use std::error::Error;
use axum::{Router, routing::{get, post}};
use store::Store;

use routes::user::{signup_handler, signin_handler , get_user};
use crate::routes::solana::{get_quote, get_sol_balance};



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("Database url must be set");

    let store = Store::connect(&db_url).await.expect("Failed to connect");

    let app = Router::new()
        .route("/api/v1/signup", post(signup_handler))
        .route("/api/v1/signin", post(signin_handler))
        .route("/api/v1/user", get(get_user))
        .route("/api/v1/sol", get(get_sol_balance))
        .route("/api/v1/quote", get(get_quote))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4444").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
