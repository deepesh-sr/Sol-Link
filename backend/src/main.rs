use axum::{Json, Router, extract::State, routing::post};
use serde::Deserialize;
use std::error::Error;
use std::net::SocketAddr;
use store::Store;

#[derive(Deserialize)]
struct SignupRequest {
    email: String,
    password: String,
    public_key: Option<String>,
}

async fn signup_handler(
    State(store): State<Store>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<store::user::User>, (axum::http::StatusCode, String)> {

    let user = store.create_user(&payload.email, &payload.password, payload.public_key.as_deref()).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let db_url = std::env::var("DATABASE_URL").expect("Database url must be set");

    let store = Store::connect(&db_url).await.expect("Failed to connect");

    let app = Router::new()
        .route("/signup", post(signup_handler))
        .with_state(store);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4444").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
