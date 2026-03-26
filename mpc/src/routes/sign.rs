use std::sync::Arc;

use axum::{Json, extract::State};
use frost_ed25519::{keys::KeyPackage, round1::SigningCommitments};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Appstate, tss};

#[derive(Deserialize)]
pub struct SignRound1Request{
    pub user_id : Uuid
}

#[derive(Serialize)]
pub struct SignRound1Response{
    pub commitments : SigningCommitments
}

pub async fn sign_round1_handler(
    State(state) : State<Arc<Appstate>>,
    Json(payload) : Json<SignRound1Request>
)->Result<Json<SignRound1Response>, (StatusCode, String)>{

    let key_package_bytes : Vec<u8>  = sqlx::query_scalar(
        r#"
            SELECT key_package FROM keyshares WHERE user_id = $1    
        "#
    ).bind(payload.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    let key_package : KeyPackage = serde_json::from_slice(&key_package_bytes).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR , e.to_string()))?;

    let ( nonce , commitments) = tss::sign_round1(key_package).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.sign_nonces.lock().await.insert(payload.user_id, nonce);


Ok(Json(SignRound1Response{commitments : commitments}))
}