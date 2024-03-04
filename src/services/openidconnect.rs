
use std::{collections::BTreeMap, sync::Arc};

use actix_web::{cookie::{Cookie, CookieJar}, get, http::header::ContentType, web, HttpRequest, HttpResponse, Responder};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Deserialize;
use sha2::Sha256;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{expose_error, storage::database::{get_current_time_seconds, Database}, structs::configuration::{Authorization, Configuration}, TOKEN_VALID_LENGTH};



#[get("/login")]
pub async fn login(configuration: web::Data<Configuration>) -> impl Responder {
    if let Authorization::OpenIdConnect {token_url: _, auth_url, revoke_url: _, userinfo_url: _, client_id, self_root_url } = &configuration.authorization {

        let verification_uuid = Uuid::new_v4().to_string();

        let cookie = Cookie::new("oidc_validation", verification_uuid.clone());

        let assembled_redirect_url = format!("{}?response_type=code&scope=openid%20profile%20email&client_id={}&state={}&redirect_uri={}auth/openidconnect", auth_url, client_id, verification_uuid, self_root_url);

        let client_redirect_html = format!("<html><head><meta http-equiv=\"refresh\" content=\"0; url='{}'\"></head><body></body></html>", assembled_redirect_url);

        HttpResponse::Ok().content_type(ContentType::html()).cookie(cookie).body(client_redirect_html)

        //Redirect::to(assembled_redirect_url).temporary()
    } else {
        HttpResponse::Ok().content_type(ContentType::html()).body("<html><h1>Invalid Authentication Configuration!</h1><html>")
    }
}

#[derive(Deserialize, Debug)]
struct RedirectParams {
    code: String,
    state: String
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
pub async fn redirect(request: HttpRequest, query: web::Query<RedirectParams>, configuration: web::Data<Configuration>, data: web::Data<Arc<Mutex<Database>>>) -> impl Responder {

    let code = &query.code;
    let state = &query.state; // Used to Verify Request Origin

    let cookie_hdr = match request.headers().get("cookie") {
        Some(cookie) =>  {
            match cookie.to_str() {
                Ok(cookie_hdr) => cookie_hdr,
                Err(_) => {
                    return HttpResponse::InternalServerError().body("{\"error\": \"failed to stringify Cookie-String!\"}");
                },
            }
        },
        None => {
            expose_error!("Failed to get Cookie set by /login Endpoint!. Invalid Request!");
        },
    };

    let cookies: Vec<&str> = cookie_hdr.split('&').collect();

    let mut valid_cookie = false;

    for cookie in cookies {
        let cook_val = match Cookie::parse_encoded(cookie) {
            Ok(cookie) => cookie,
            Err(_err) => {
                expose_error!("Failed to deserialize Cookie!");
            },
        };

        if cook_val.name().eq("oidc_validation") && cook_val.value().eq(state) {
            valid_cookie = true;
        }
    }

    if !valid_cookie {
            expose_error!("Failed to verify state! Invalid Request!");
    }

    let mut cookie_jar = CookieJar::new();

    cookie_jar.remove(Cookie::named("oidc_validation"));

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
