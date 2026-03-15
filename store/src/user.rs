use serde::{Deserialize,Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::FromRow;

#[derive( Serialize,Debug, FromRow)]
pub struct User {
    id :Uuid ,
    email : String,
    #[serde(skip_serializing)]
    password : String, 
    created_at : Option<DateTime<Utc>>,
    updated_at : Option<DateTime<Utc>>,
    public_key : Option<String>
}


#[derive(Serialize,Debug, FromRow)]
pub struct Asset {
    id :Uuid ,
    mint_address : String, 
    decimals : i32,
    name : String, 
    symbol : String, 
    logo_url : Option<String>, 
    created_at : Option<DateTime<Utc>>,
    updated_at : Option<DateTime<Utc>>
}

#[derive(Serialize,Debug, FromRow)]
pub struct Balance { 
    id : Uuid, 
    amount : i64, 
    created_at : Option<DateTime<Utc>>,
    updated_at : Option<DateTime<Utc>>,
    user_id : Uuid,
    asset_id : Uuid
}