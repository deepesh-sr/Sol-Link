use std::{collections::BTreeMap, sync::Arc};

use axum::{Json, extract::State};
use frost_ed25519::{Identifier, keys::{KeyPackage, PublicKeyPackage, dkg::{round1, round2}}};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{Appstate, tss};

#[derive(Serialize)]
pub struct DkgRound1Response {
    pub round1_package: serde_json::Value,
}

#[derive(Deserialize)]
pub struct DkgRound1Request {
    user_id: Uuid,
}

pub async fn dkg_route1_handler(
    State(state): State<Arc<Appstate>>,
    Json(payload): Json<DkgRound1Request>,
) -> Result<Json<DkgRound1Response>, (StatusCode, String)> {
    let (secret, package) = tss::dkg_part1(state.node_id)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state
        .dkg_round1_secrets
        .lock()
        .await
        .insert(payload.user_id, secret);

    let package_json = serde_json::to_value(&package)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(DkgRound1Response {
        round1_package: package_json,
    }))
}

#[derive(Deserialize)]
pub struct DkgRound2Request{
    pub user_id : Uuid, 
    pub round1_packages : BTreeMap<Identifier, round1::Package>
}

#[derive(Serialize)]
pub struct DkgRound2Response{
    pub round2_package : serde_json::Value
}

pub async fn dkg_route2_handler(
    State(state) : State<Arc<Appstate>>,
    Json(payload) : Json<DkgRound2Request>
)-> Result<Json<DkgRound2Response>, ( StatusCode, String)>{

    let secret = state.dkg_round1_secrets.lock().await.remove(&payload.user_id).ok_or((StatusCode::BAD_REQUEST, "No round1 secret found for round 1".to_string()))?;

    let (round2_secret, round2_package) = tss::dkg_part2(secret, &payload.round1_packages).map_err(|e|(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    state.dkg_round2_secrets.lock().await.insert(payload.user_id, round2_secret);

    let round2_package_json = serde_json::to_value(&round2_package).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(DkgRound2Response {
        round2_package: round2_package_json,
    }))
}



#[derive(Deserialize)]
pub struct DkgRound3Request{
    pub user_id : Uuid, 
            round1_packages : BTreeMap<Identifier, round1::Package>,

    pub round2_packages : BTreeMap<Identifier, round2::Package>
}

#[derive(Serialize)]
pub struct DkgRound3Response{
    pub public_key_package: serde_json::Value,
}

pub async fn dkg_round3_handler(
    State(state) : State<Arc<Appstate>>,
    Json(payload) : Json<DkgRound3Request>
)-> Result<Json<DkgRound3Response>, (StatusCode, String)>{
    let round2_secret = state.dkg_round2_secrets.lock().await.remove(&payload.user_id)
        .ok_or((StatusCode::BAD_REQUEST, "No round2 secret found".to_string()))?;

    let (key_package, public_key_package) = tss::dkg_part3(&round2_secret, &payload.round1_packages, &payload.round2_packages)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    let key_package_bytes = serde_json::to_vec(&key_package)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let pubkey_package_bytes = serde_json::to_vec(&public_key_package)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    sqlx::query(
        "INSERT INTO keyshares (user_id, key_package, public_key_package) VALUES ($1, $2, $3)"
    )
        .bind(payload.user_id)
        .bind(&key_package_bytes)
        .bind(&pubkey_package_bytes)
        .execute(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;


    let pubkey_json = serde_json::to_value(&public_key_package)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(DkgRound3Response {
        public_key_package: pubkey_json,
    }))
}
