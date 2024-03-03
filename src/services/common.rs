use std::{collections::BTreeMap, sync::Arc};

use actix_web::web;
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use tokio::sync::Mutex;

use crate::storage::database::{get_current_time_seconds, Database};

pub async fn authenticate(token: &str, data: web::Data<Arc<Mutex<Database>>>, token_secret: String) -> Result<(bool, Option<String>), String> { // authenticated, email
    let token_key: Hmac<Sha256> = match Hmac::new_from_slice(token_secret.as_bytes()) {
        Ok(token) => token,
        Err(err) => {
            return Err(format!("Failed to create HMAC!: {:?}", err));
        },
    };

    let claims: BTreeMap<String, String> = match token.verify_with_key(&token_key) {
        Ok(claims) => claims,
        Err(err) => {
            return Err(format!("Authentication Failed: {:?}", err));
        },
    };

    let expiry_time = match claims.get("exp") {
        Some(time) => {
            match time.parse::<u64>() {
                Ok(result) => result,
                Err(err) => {
                    return Err(format!("Was presented with malformed Token: {:?}", err));
                },
            }
        },
        None => {
            return Err("Was presented with malformed Token!".to_string());
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
            drop(database);
        });
        return Ok((false, None));
    }

    let uuid = match claims.get("sessionid") {
        Some(id) => id,
        None => {
            return Err("Malformed Token!".to_string());
        },
    };

    let mail = match claims.get("sub") {
        Some(id) => id,
        None => {
            return Err("Malformed Token!".to_string());
        },
    };


    let mut database = data.lock().await;

    let valid = match database.is_session_valid(uuid) {
        Ok(valid) => valid,
        Err(err) => {
            return Err(format!("Failed to fetch Mail from UUID: {:?}", err));
        },
    };

    Ok((valid, Some(mail.to_string())))

}

pub async fn authenticate_admin(token: &str, data: web::Data<Arc<Mutex<Database>>>, token_secret: String) -> Result<(bool, Option<String>), String> {
    let auth = match authenticate(token, data.clone(), token_secret).await {
        Ok(auth) => auth,
        Err(err) => return Err(err),
    };

    if !auth.0 {
        return Ok((false, None));
    }

    let mail = match auth.1 {
        Some(mail) => mail,
        None => {
            return Err("No Mail in validated Request!".to_string());
        },
    };

    let mut database = data.lock().await;

    match database.check_if_user_admin(&mail) {
        Ok(admin) => {
            Ok((admin, Some(mail)))
        },
        Err(err) => {
            Err(format!("Failed to get Admin Status from Database!: {:?}", err))
        },
    }
}

#[macro_export]
macro_rules! authenticate {
    ($request:expr, $data:expr, $encryption_secret:expr) => {
        let auth_header = match $request.headers().get("Authorization") {
            Some(header) => header,
            None => {
                invalid_input!("Missing Authentication Header!");
            },
        };

        let mut token = match auth_header.to_str() {
            Ok(header) => header.to_string(),
            Err(err) => {
                invalid_input!(&format!("Missing Authentication Header!: {:?}", err));
            },
        };

        token = token.replace("Bearer ", "");

        match authenticate(&token, $data.clone(), $encryption_secret).await {
            Ok((valid, _)) => {
                if !valid {
                    return HttpResponse::Forbidden().content_type(ContentType::json()).body("{\"error\":\"Invalid Credentials\"}");
                }
            },
            Err(err) => {
                invalid_input!(&format!("Failed to Authenticate!: {:?}", err));
            },
        }
    };
}



#[macro_export]
macro_rules! authenticate_admin {
    ($request:expr, $data:expr, $encryption_secret:expr) => {
        let auth_header = match $request.headers().get("Authorization") {
            Some(header) => header,
            None => {
                invalid_input!("Missing Authentication Header!");
            },
        };

        let mut token = match auth_header.to_str() {
            Ok(header) => header.to_string(),
            Err(err) => {
                invalid_input!(&format!("Missing Authentication Header!: {:?}", err));
            },
        };

        token = token.replace("Bearer ", "");

        match authenticate_admin(&token, $data.clone(), $encryption_secret).await {
            Ok((valid, _)) => {
                if !valid {
                    return HttpResponse::Forbidden().content_type(ContentType::json()).body("{\"error\":\"Invalid Credentials\"}");
                }
            },
            Err(err) => {
                invalid_input!(&format!("Failed to Authenticate!: {:?}", err));
            },
        }
    };
}
