use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use store::Store;
use bcrypt::{hash,DEFAULT_COST};

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    public_key: Option<String>,
}

pub async fn signup_handler(
    State(store): State<Store>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<store::user::User>, (axum::http::StatusCode, String)> {

    let hashed_password = hash(payload.password , DEFAULT_COST).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?;

    let user = store.create_user(&payload.email, &hashed_password, payload.public_key.as_deref()).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}
