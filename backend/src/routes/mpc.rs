use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub async  fn orchestrate_dkg(user_id : Uuid)-> Result<String, reqwest::Error>{

    let client = reqwest::Client::new();

    let nodes = ["http://localhost:5001", "http://localhost:5002", "http://localhost:5003"];

    let pkg1 : serde_json::Value = client.post(format!("{}/dkg/round1",nodes[0]))
                                .json(&json!({"user_id" : user_id}))
                                .send().await?.json().await?;

    let pkg2 : serde_json::Value = client.post(format!("{}/dkg/round1",nodes[1]))
                                .json(&json!({"user_id" : user_id}))
                                .send().await?.json().await?;

    let pkg3 : serde_json::Value = client.post(format!("{}/dkg/round1",nodes[2]))
                                .json(&json!({"user_id" : user_id}))
                                .send().await?.json().await?;


    let round2_res1: serde_json::Value = client.post(format!("{}/dkg/round2", nodes[0]))
        .json(&json!({
            "user_id": user_id,
            "round1_packages": { "2": pkg2["round1_package"], "3": pkg3["round1_package"] }
        }))
        .send().await?.json().await?;

    let round2_res2: serde_json::Value = client.post(format!("{}/dkg/round2", nodes[1]))
        .json(&json!({
            "user_id": user_id,
            "round1_packages": { "1": pkg1["round1_package"], "3": pkg3["round1_package"] }
        }))
        .send().await?.json().await?;

    let round2_res3: serde_json::Value = client.post(format!("{}/dkg/round2", nodes[2]))
        .json(&json!({
            "user_id": user_id,
            "round1_packages": { "1": pkg1["round1_package"], "2": pkg2["round1_package"] }
        }))
        .send().await?.json().await?;

    let round3_res1: serde_json::Value = client.post(format!("{}/dkg/round3", nodes[0]))
        .json(&json!({
            "user_id": user_id,
            "round1_packages": { "2": pkg2["round1_package"], "3": pkg3["round1_package"] },
            "round2_packages": { "2": round2_res2["round2_package"], "3": round2_res3["round2_package"] }
        }))
        .send().await?.json().await?;

    let _round3_res2: serde_json::Value = client.post(format!("{}/dkg/round3", nodes[1]))
        .json(&json!({
            "user_id": user_id,
            "round1_packages": { "1": pkg1["round1_package"], "3": pkg3["round1_package"] },
            "round2_packages": { "1": round2_res1["round2_package"], "3": round2_res3["round2_package"] }
        }))
        .send().await?.json().await?;

    let _round3_res3: serde_json::Value = client.post(format!("{}/dkg/round3", nodes[2]))
        .json(&json!({
            "user_id": user_id,
            "round1_packages": { "1": pkg1["round1_package"], "2": pkg2["round1_package"] },
            "round2_packages": { "1": round2_res1["round2_package"], "2": round2_res2["round2_package"] }
        }))
        .send().await?.json().await?;


        let public_key = round3_res1["public_key_package"].to_string();

    Ok(public_key)
}

pub async fn orchestrate_signing(user_id : Uuid , tx_bytes : Vec<u8>)-> Result<String, reqwest::Error>{
    
    let client = Client::new();
 let nodes = ["http://localhost:5001", "http://localhost:5002", "http://localhost:5003"];

    let commitment_res1 : serde_json::Value = client.post(format!("{}/sign/round1",nodes[0]))
                                        .json(&json!({
                                            "user_id" : user_id,
                                        }))
                                        .send().await?
                                        .json().await?;
                                    
    let commitment_res2 : serde_json::Value = client.post(format!("{}/sign/round1",nodes[1]))
                                        .json(&json!({
                                            "user_id" : user_id,
                                        }))
                                        .send().await?
                                        .json().await?;
  
    // SigningPackage = commitments + message (tx_bytes)

let signing_pkg_res: serde_json::Value = client.post(format!("{}/sign/create-signing-package", nodes[0]))
    .json(&json!({
        "commitments_map": {
            "1": commitment_res1["commitments"],
            "2": commitment_res2["commitments"]
        },
        "message": tx_bytes
    }))
    .send().await?.json().await?;


// round 2 signing 
let share1: serde_json::Value = client.post(format!("{}/sign/round2", nodes[0]))
    .json(&json!({
        "user_id": user_id,
        "signing_package": signing_pkg_res
    }))
    .send().await?.json().await?;

let share2: serde_json::Value = client.post(format!("{}/sign/round2", nodes[1]))
    .json(&json!({
        "user_id": user_id,
        "signing_package": signing_pkg_res
    }))
    .send().await?.json().await?;
                                        
    
    let aggregate_res: serde_json::Value = client.post(format!("{}/sign/aggregate", nodes[0]))
    .json(&json!({
        "user_id": user_id,
        "signing_package": signing_pkg_res["signing_package"],
        "signature_shares": {
            "1": share1["signature_share"],
            "2": share2["signature_share"]
        }
    }))
    .send().await?.json().await?;

Ok(aggregate_res["signature"].to_string())

}