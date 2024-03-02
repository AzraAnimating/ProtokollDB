
use actix_web::{body, get, http::header::ContentType, web::{self, Redirect}, HttpRequest, HttpResponse, Responder};
use serde::Deserialize;

use crate::{expose_error, structs::configuration::{Authorization, Configuration}};



#[get("/login")]
pub async fn login(configuration: web::Data<Configuration>) -> impl Responder {
    if let Authorization::OpenIdConnect {token_url: _, auth_url, logout_url: _, userinfo_url: _, client_id, self_root_url } = &configuration.authorization {
        let assembled_redirect_url = format!("{}?response_type=code&scope=openid%20profile%20roles%20email&client_id={}&redirect_uri={}auth/openidconnect", auth_url, client_id, self_root_url);
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

#[get("/auth/openidconnect")]
pub async fn redirect(query: web::Query<RedirectParams>, configuration: web::Data<Configuration>) -> impl Responder {

    let code = &query.code;

    let client = reqwest::Client::new(); 


    if let Authorization::OpenIdConnect {token_url, auth_url, logout_url, userinfo_url, client_id, self_root_url } = &configuration.authorization {

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
        let response = match serde_json::from_str::<TokenResponse>(&response_str) {
            Ok(response) => response,
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };

        // Here, we override the Variables for response with the Response for /userinfo request.

        let response = match client.get(userinfo_url).bearer_auth(response.access_token).send().await {
            Ok(response) => response,
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };


        let response_bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(err) => {
                expose_error!(&format!("{:?}", err));
            },
        };

        let response_str = String::from_utf8_lossy(&response_bytes).to_string();

        HttpResponse::Ok().body(format!("{}", response_str))

    } else {
        HttpResponse::Unauthorized().body("{\"error\":\"Authorization isn't set to openidconnect!'\"}")
    }
    


    //HttpResponse::Ok().body(format!("{:?}", query))
}

#[get("/auth/openidconnect/done")]
pub async fn finish() -> impl Responder {
    HttpResponse::Ok().body("")
}
