use std::{collections::BTreeMap, sync::Arc};

use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use tokio::sync::Mutex;

use crate::storage::database::{get_current_time_seconds, Database};

pub async fn authenticate(token: String, data: Arc<Mutex<Database>>, token_secret: String) -> Option<(bool, bool)> { // authenticated, isAdmin
    let token_key: Hmac<Sha256> = match Hmac::new_from_slice(token_secret.as_bytes()) {
        Ok(token) => token,
        Err(err) => {
            print!("Failed to create HMAC!: {:?}", err); 
            return None
        },
    };

    let claims: BTreeMap<String, String> = match token.verify_with_key(&token_key) {
        Ok(claims) => claims,
        Err(err) => {
            println!("Authentication Failed: {:?}", err); 
            return None;
        },
    };

    let expiry_time = match claims.get("exp") {
        Some(time) => {
            match time.parse::<u64>() {
                Ok(result) => result,
                Err(err) => {
                    println!("Was presented with malformed Token: {:?}", err); 
                    return None;
                },
            }
        },
        None => {
            println!("Was presented with malformed Token"); 
            return None;
        },
    };

    //Every Time someone connects with a expired session we remove all the other sessions from the
    //Database
    if get_current_time_seconds() > expiry_time {
        tokio::spawn(async move {
            let mut database = data.lock().await;
            match database.remove_expired_sessions() {
                Ok(_) => {},
                Err(err) => {
                    println!("Failed to remove expired Sessions!: {:?}", err); 
                },
            }
        });
        return Some((false, false));//Not Authenticated because token expired some time ago
    }

    None
}
