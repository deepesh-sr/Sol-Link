use std::{collections::BTreeMap, sync::Arc};

use axum::{Json, extract::State};
use frost_ed25519::{Identifier, keys::dkg::round1};
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

    let round2_package_json = serde_json::to_value(&round2_package).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(DkgRound2Response {
        round2_package: round2_package_json,
    }))
}
