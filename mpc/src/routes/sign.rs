use std::{collections::BTreeMap, sync::Arc};

use axum::{Json, extract::State};
use frost_ed25519::{Identifier, SigningPackage, keys::{KeyPackage, PublicKeyPackage}, round1::SigningCommitments, round2::SignatureShare};
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

#[derive(Deserialize)]
pub struct SignRound2Request{
    pub user_id : Uuid,
    pub signing_package : SigningPackage
}

#[derive(Serialize)]
pub struct SignRound2Response{
    signature_share : SignatureShare
}
pub async fn sign_round2_handler(
    State(state) : State<Arc<Appstate>>,
    Json(payload) : Json<SignRound2Request>
)-> Result<Json<SignRound2Response>,(StatusCode,String)>{

    let sign_nonces = state.sign_nonces.lock().await.remove(&payload.user_id).ok_or((StatusCode::BAD_REQUEST, "No round1 secret found for round 1".to_string()))?;
    let key_package_bytes : Vec<u8>  = sqlx::query_scalar(
        r#"
            SELECT key_package FROM keyshares WHERE user_id = $1    
        "#
    ).bind(payload.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    let key_package : KeyPackage = serde_json::from_slice(&key_package_bytes).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR , e.to_string()))?;

   let signature_share=  tss::sign_round2(payload.signing_package, &sign_nonces, &key_package).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR , e.to_string()))?;

    Ok(Json(SignRound2Response{signature_share : signature_share}))
}

#[derive(Deserialize)]
pub struct SignPackageRequest {
    pub commitments_map: BTreeMap<Identifier, SigningCommitments>,
    pub message: Vec<u8>,
}

#[derive(Serialize)]
pub struct SignPackageResponse {
    pub signing_package: serde_json::Value,
}

pub async fn create_signing_package_handler(
    Json(payload): Json<SignPackageRequest>,
) -> Result<Json<SignPackageResponse>, (StatusCode, String)> {
    
    let signing_package = frost_ed25519::SigningPackage::new(
        payload.commitments_map,
        &payload.message,
    );
    
    let sp_json = serde_json::to_value(&signing_package)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(SignPackageResponse { signing_package: sp_json }))
}


#[derive(Deserialize)]
pub struct AggregateRequest {
    pub user_id: Uuid,
    pub signing_package: SigningPackage,
    pub signature_shares: BTreeMap<Identifier, SignatureShare>,
}

#[derive(Serialize)]
pub struct AggregateResponse {
    pub signature: serde_json::Value,
}

pub async fn aggregate_handler(
    State(state) : State<Arc<Appstate>>,
    Json(payload) : Json<AggregateRequest>
)-> Result<Json<AggregateResponse>,(StatusCode,String)>{

    let public_key_package_bytes : Vec<u8>  = sqlx::query_scalar(
        r#"
            SELECT public_key_package FROM keyshares WHERE user_id = $1    
        "#
    ).bind(payload.user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

   let pubkey_package : PublicKeyPackage = serde_json::from_slice(&public_key_package_bytes).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR , e.to_string()))?;

   let signature = tss::aggregate(payload.signing_package, &payload.signature_shares, pubkey_package)
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

   let sig_json = serde_json::to_value(&signature)
       .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

   Ok(Json(AggregateResponse { signature: sig_json }))
}
