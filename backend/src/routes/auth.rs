use axum::{extract::FromRequestParts, http::StatusCode};
use jsonwebtoken::{Algorithm,Validation, DecodingKey};
use store::Store;

use crate::routes::Claims;


impl FromRequestParts<Store> for Claims {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
            parts: &mut axum::http::request::Parts,
            state: &Store,
        ) -> Result<Self, Self::Rejection> {
            
        let secret = std::env::var("JWT_SECRET").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR,"JWT_SECRET not set".to_string()))?;

        let auth_header = parts.headers
            .get("Authorization")
            .ok_or((StatusCode::UNAUTHORIZED,"Mising Token".to_string()))?
            .to_str()
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid Header".to_string()))?;

       let token_data = jsonwebtoken::decode::<Claims>(
            &auth_header,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::new(Algorithm::HS256)
            ).map_err(|_| (StatusCode::UNAUTHORIZED , "Invalid token".to_string()))?;
        
        Ok(token_data.claims)
    }     
}