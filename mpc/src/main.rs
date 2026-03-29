use std::{collections::HashMap, env, sync::Arc};
use tokio::sync::Mutex;

use axum::{Router, routing::post};
use frost_ed25519::{Identifier, keys::dkg, round1::SigningNonces};

use dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio;
use uuid::Uuid;

pub mod routes;
pub mod tss;

use routes::dkg::{dkg_route1_handler, dkg_route2_handler, dkg_round3_handler};
use routes::sign::{sign_round1_handler,sign_round2_handler};

use crate::routes::sign::{aggregate_handler, create_signing_package_handler};


pub struct Appstate {
    node_id: Identifier,
    dkg_round1_secrets: Mutex<HashMap<Uuid, dkg::round1::SecretPackage>>,
    dkg_round2_secrets: Mutex<HashMap<Uuid, dkg::round2::SecretPackage>>,
    sign_nonces: Mutex<HashMap<Uuid, SigningNonces>>,
    db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let node_id: u16 = env::var("NODE_ID").unwrap().parse().unwrap();
    let identifier = Identifier::try_from(node_id).unwrap();
    let db_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await?;
    let port = env::var("PORT").unwrap_or("5001".to_string());

    let state = Arc::new(Appstate {
        node_id: identifier,
        dkg_round1_secrets: Mutex::new(HashMap::new()),
        dkg_round2_secrets: Mutex::new(HashMap::new()),
        sign_nonces: Mutex::new(HashMap::new()),
        db: pool,
    });

    let app = Router::new()
        .route("/dkg/round1", post(dkg_route1_handler))
        .route("/dkg/round2", post(dkg_route2_handler))
        .route("/dkg/round3", post(dkg_round3_handler))
        .route("/sign/round1", post(sign_round1_handler))
        .route("/sign/round2", post(sign_round2_handler))
        .route("/sign/create-signing-package", post(create_signing_package_handler))
        .route("/sign/aggregate", post(aggregate_handler))
        .with_state(state);

    let address = format!("0.0.0.0:{}",port);
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
