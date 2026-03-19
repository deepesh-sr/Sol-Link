use std::str::FromStr;

use axum::{Json, extract::{Query, State}, http::StatusCode};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{lamports, pubkey::{self, Pubkey}};
use store::Store;

use crate::routes::Claims;

#[derive(Deserialize)]
pub struct QuoteRequest{
    pub input_mint : String, 
    pub output_mint : String, 
    pub amount : u64, 
    pub slippage_bps : Option<u16>
}

#[derive(Serialize)]
pub struct BalanceResponse{
    pub sol_balance : f64,
}

pub async fn get_sol_balance(
    State(store) : State<Store>,
    claims : Claims
)-> Result<Json<BalanceResponse>,( StatusCode, String)>{

    let uuid = claims.get_sub().map_err(|e| (StatusCode::BAD_REQUEST,e))?;
    let user = store.get_user_by_id(&uuid)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR ,e.to_string()))?;

    let pubkey_str = user.get_pubkey().ok_or((StatusCode::BAD_REQUEST, "No public key associated wiht this account".to_string()))?;


    let rpc_url = std::env::var("SOLANA_RPC").expect("RPC url must be there");
    let client = RpcClient::new(rpc_url);

    // fetch balance 
    let pubkey = Pubkey::from_str(&pubkey_str).map_err(|_| ( StatusCode::BAD_REQUEST, "Invalid public key format".to_string()))?;

    let lamports = client.get_balance(&pubkey).map_err(|e| ( StatusCode::INTERNAL_SERVER_ERROR , e.to_string()))?;

    let sol_balance = lamports as f64 / 1_000_000_000.0;

    Ok(Json(BalanceResponse { sol_balance }))
}

pub async fn get_quote(Query(params) : Query<QuoteRequest>)-> Result<Json<serde_json::Value>, (StatusCode , String)> {

    let api_key = std::env::var("JUPITER_API_KEY")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JUPITER_API_KEY not set".to_string()))?;

    // Jupiter Quote API URL
    let url = format!(
        "https://api.jup.ag/swap/v1/quote?inputMint={}&outputMint={}&amount={}&slippageBps={}",
        params.input_mint,
        params.output_mint,
        params.amount,
        params.slippage_bps.unwrap_or(50) // Default 0.5% slippage
    );

    let client = reqwest::Client::new();
    let response = client.get(&url)
        .header("x-api-key", &api_key)
        .send()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .json::<serde_json::Value>()
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(response))
}