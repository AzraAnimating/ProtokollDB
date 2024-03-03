
use std::{collections::BTreeMap, sync::Arc};

use actix_web::{get, http::header::ContentType, web::{self, Redirect}, HttpResponse, Responder};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Deserialize;
use sha2::Sha256;
use tokio::sync::Mutex;

use crate::{expose_error, storage::database::{get_current_time_seconds, Database}, structs::configuration::{Authorization, Configuration}, TOKEN_VALID_LENGTH};



#[get("/login")]
pub async fn login(configuration: web::Data<Configuration>) -> impl Responder {
    if let Authorization::OpenIdConnect {token_url: _, auth_url, revoke_url: _, userinfo_url: _, client_id, self_root_url } = &configuration.authorization {
        let assembled_redirect_url = format!("{}?response_type=code&scope=openid%20profile%20email&client_id={}&redirect_uri={}auth/openidconnect", auth_url, client_id, self_root_url);
        Redirect::to(assembled_redirect_url).temporary()
    } else {
        Redirect::to("/invalidauth").temporary()
    }
}

#[derive(Deserialize, Debug)]
struct RedirectParams {
    code: String
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String
}

#[derive(Deserialize, Debug)]
struct UserInfo {
    email: String
}

#[get("/auth/openidconnect")]
pub async fn redirect(query: web::Query<RedirectParams>, configuration: web::Data<Configuration>, data: web::Data<Arc<Mutex<Database>>>) -> impl Responder {

    let code = &query.code;

    let client = reqwest::Client::new();


    if let Authorization::OpenIdConnect {token_url, auth_url:_, revoke_url, userinfo_url, client_id, self_root_url } = &configuration.authorization {

        let mut map = vec![];

        map.push(("grant_type".to_string(), "authorization_code".to_string()));
        map.push(("code".to_string(), code.to_string()));
        map.push(("client_id".to_string(), client_id.to_string()));
        map.push(("redirect_uri".to_string(), format!("{}auth/openidconnect", self_root_url)));

                                                    
        let response = match client.post(token_url.clone()).form(&map).send().await {
            Ok(response) => response, 
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            }
        };

        let response_bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };

        let response_str = String::from_utf8_lossy(&response_bytes).to_string();
        let token_response = match serde_json::from_str::<TokenResponse>(&response_str) {
            Ok(response) => response,
            Err(err) => {
                expose_error!(&format!("Failed to deserialize Response: {:?}", err));
            },
        };

        // Here, we override the Variables for response with the Response for /userinfo request.

        let response = match client.get(userinfo_url).bearer_auth(&token_response.access_token).send().await {
            Ok(response) => response,
            Err(err) => {
                expose_error!(&format!("Failed to get Userinfo: {:?}", err));
            },
        };


        let map = vec![("client_id".to_string(), client_id.to_string()), ("token".to_string(), token_response.access_token.clone())];

        match client.post(revoke_url).form(&map).send().await {
            Ok(_) => {},
            Err(err) => {
                expose_error!(&format!("Failed to Revoke Token!: {:?}", err));
            },
        };


        let response_bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                expose_error!(&format!("Failed to Bytify Response: {:?}", err));
            },
        };

        let response_str = String::from_utf8_lossy(&response_bytes).to_string();
        let response = match serde_json::from_str::<UserInfo>(&response_str) {
            Ok(response) => response,
            Err(err) => {
                expose_error!(&format!("Failed to construct Userdata{:?}", err));
            },
        };


        let mut database = data.lock().await;

        
        let uuid = match database.save_access_token() {
            Ok(uuid) => {
                match uuid {
                    Some(uuid) => uuid,
                    None => {
                        return HttpResponse::InternalServerError().content_type(ContentType::json()).body("{\"error\":\"failed to get new UUID\"}")
                    },
                }
            },
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };

        drop(database);

        
        let mut claims = BTreeMap::new();

        claims.insert("sub", response.email.clone());
        claims.insert("iss", "ProtocolDB".to_string());
        claims.insert("exp", format!("{}", get_current_time_seconds() + TOKEN_VALID_LENGTH));
        claims.insert("sessionid", uuid);

        let token_key: Hmac<Sha256> = match Hmac::new_from_slice(configuration.encryption.token_encryption_secret.clone().as_bytes()) {
            Ok(token) => token,
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };

        let token_str = match claims.sign_with_key(&token_key) {
            Ok(token) => token,
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };

        //Todo Redirect to frontend 

        HttpResponse::Ok().body(token_str)
    } else { //These values are Returned, because rust returns when there is no trailing semicolon
        HttpResponse::Unauthorized().body("{\"error\":\"Authorization isn't set to openidconnect!'\"}")
    }
    //HttpResponse::Ok().body(format!("{:?}", query))
}

#[get("/auth/openidconnect/done")]
pub async fn finish() -> impl Responder {
    HttpResponse::Ok().body("")
}
