use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use store::Store;
use bcrypt::{DEFAULT_COST, hash, verify};

#[derive(Deserialize)]
pub struct SignupRequest {
    email: String,
    password: String,
    public_key: Option<String>,
}

#[derive(Deserialize)]
pub struct SigninRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct SigninResponse{
    token : String
}


#[derive(Deserialize,Serialize)]
pub struct Claims{
    sub : String, // for user_id
    exp : usize // expiry check krne k liye ( unix timestamp )
}

pub async fn signup_handler(
    State(store): State<Store>,
    Json(payload): Json<SignupRequest>,
) -> Result<Json<store::user::User>, (axum::http::StatusCode, String)> {

    let hashed_password = hash(payload.password , DEFAULT_COST).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password".to_string()))?;

    let user = store.create_user(&payload.email, &hashed_password, payload.public_key.as_deref()).await.map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(user))
}

pub async fn signin_handler(
    State(store) : State<Store>,
    Json(payload) : Json<SigninRequest>
)-> Result<Json<SigninResponse>,(axum::http::StatusCode, String)>{

    let user = store.get_user_by_email(&payload.email).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    verify(payload.password, user.get_password()).map_err(|_| (StatusCode::UNAUTHORIZED,"Incorrect Password".to_string()))?;

    let claims = Claims{
        sub : user.get_id().to_string(),
        exp : (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };

    let secret = std::env::var("JWT_SECRET").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT secret not set".to_string()))?;

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token genration failed".to_string()))?;


Ok(Json(SigninResponse{token}))

}