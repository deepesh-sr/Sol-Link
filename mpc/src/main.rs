use std::{collections::HashMap, sync::Mutex};

use frost_ed25519::{Identifier, keys::dkg, round1::SigningNonces};
use sqlx::PgPool;
use uuid::Uuid;

pub mod tss;
pub mod routes;


struct Appstate{

    node_id : Identifier,
    dkg_round1_secrets : Mutex<HashMap<Uuid , dkg::round1::SecretPackage>>,
    dkg_round2_secrets : Mutex<HashMap<Uuid , dkg::round2::SecretPackage>>,
    sign_nonces : Mutex<HashMap<Uuid, SigningNonces>>,
    db : PgPool
}
fn main() {


}
