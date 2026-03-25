use std::sync::Arc;

use axum::{Json, extract::State};
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
