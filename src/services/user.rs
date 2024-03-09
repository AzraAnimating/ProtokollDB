use std::{fs, num::ParseIntError, sync::Arc};

use actix_web::{get, http::header::ContentType, post, web::{self, Query}, HttpRequest, HttpResponse, Responder};
use tokio::sync::Mutex;

use crate::{authenticate, expose_error, invalid_input, storage::database::{get_current_time_seconds, Database}, structs::{configuration::Configuration, get_inputs::Search, post_inputs::SubmittingProtocol, submitted_protocol::SubmittedProtocol}};

use super::common::authenticate;


#[get("/api/v1/identifiers")]
async fn get_selection_identifiers(request: HttpRequest, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {

    authenticate!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

    let database = data.lock().await;

    let identifiers = match database.get_selection_identifiers() {
        Ok(idents) => idents,
        Err(err) => {
            expose_error!(&format!("Failed to fetch Selection IDs!: {:?}", err));
        },
    };

    drop(database);

    HttpResponse::Ok().content_type(ContentType::json()).json(identifiers)
}

#[post("/api/v1/submit")]
async fn submit_protocol(request: HttpRequest, protocol: web::Json<SubmittingProtocol>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {
    
    let auth_header = match request.headers().get("Authorization") {
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

    let (_, potential_addr) = match authenticate(&token, data.clone(), configuration.encryption.token_encryption_secret.clone()).await {
        Ok((valid, addr)) => {
            if !valid {
                return HttpResponse::Forbidden().content_type(ContentType::json()).body("{\"error\":\"Invalid Credentials\"}");
            }
            (valid, addr)
        },
        Err(err) => {
            invalid_input!(&format!("Failed to Authenticate!: {:?}", err));
        },
    };
    
    let addr = match potential_addr {
        Some(addr) => addr,
        None => {
            invalid_input!("Failed to get registration-address");
        },
    };

    let submitted_protocol = SubmittedProtocol { author: addr, subject_examiners: protocol.examiner_subjects.clone(), grades: protocol.grades.clone(), stex: protocol.stex, year: protocol.year, season: protocol.season, hand_in_date: get_current_time_seconds() };

    let submit_protocol_str = match serde_json::to_string(&submitted_protocol) {
        Ok(protocol) => protocol,
        Err(err) => {
            expose_error!(&format!("Failed to serialize Protocol!: {:?}", err));
        },
    };

    let mut database = data.lock().await;

    let potential_uuid = match database.save_submitted_protocol() {
        Ok(uuid) => uuid,
        Err(err) => {
            expose_error!(&format!("Failed to save Submitted Protocol!: {:?}", err));
        },
    };

    let uuid = match potential_uuid {
        Some(uuid) => uuid,
        None => {
            expose_error!("Failed to get save Submitted Protocol!");
        },
    };

    match fs::write(format!("submitted_protocols/{}.json", uuid), submit_protocol_str) {
        Ok(_) => {
            HttpResponse::Ok().body("")
        },
        Err(err) => {
            expose_error!(&format!("Failed to write submitted Protocol to Disk!: {:?}", err))
        },
    }
}


#[get("/api/v1/search")]
async fn search_for_protocol(request: HttpRequest, search_terms: Query<Search>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {

    authenticate!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());
    

    let subjects = match parse_input_to_id_vec(&search_terms.subjects) {
        Ok(val) => val,
        Err(err) => {
            invalid_input!(&err.to_string());
        },
    };

    let examiners = match parse_input_to_id_vec(&search_terms.examiners) {
        Ok(val) => val,
        Err(err) => {
            invalid_input!(&err.to_string());
        },
    };

    let stex = match parse_input_to_id_vec(&search_terms.stex) {
        Ok(val) => val,
        Err(err) => {
            invalid_input!(&err.to_string());
        },
    };

    let seasons = match parse_input_to_id_vec(&search_terms.seasons) {
        Ok(val) => val,
        Err(err) => {
            invalid_input!(&err.to_string());
        },
    };

    let years = match parse_input_to_id_vec(&search_terms.years) {
        Ok(val) => val,
        Err(err) => {
            invalid_input!(&err.to_string());
        },
    };

    if examiners.is_none() && subjects.is_none() && stex.is_none() && seasons.is_none() && years.is_none() {
        return HttpResponse::NotFound().content_type(ContentType::json()).body("{\"error\":\"No Search Parameters Provided\"}");
    }
    
    let database = data.lock().await;

    let potential_results = match database.search_for_protocol(examiners, subjects, stex,seasons, years) {
        Ok(results) => results,
        Err(err) => {
            expose_error!(&err.to_string());
        },
    };

    let results = match potential_results {
        Some(results) => results,
        None => return HttpResponse::NotFound().content_type(ContentType::json()).body("{\"error\":\"Found no protocols matching provided Parameters\"}"),
    };

    let serialized_return_val = match serde_json::to_string(&results) {
        Ok(val) => val,
        Err(err) => {
            expose_error!(&err.to_string());
        },
    };

    HttpResponse::Ok().content_type(ContentType::json()).body(serialized_return_val)
}



fn parse_input_to_id_vec(input: &Option<String>) -> Result<Option<Vec<i64>>, ParseIntError> {
    match input {
        Some(id_str) => {
            let split = id_str.split(',');
            let mut assembled_vec = vec![];

            for element in split {
                let num = match element.parse::<i64>() {
                    Ok(number) => number, 
                    Err(err) => {
                        return Result::Err(err)
                    }
                };
                assembled_vec.push(num)
            };

            Result::Ok(Some(assembled_vec))
        },
        None => Result::Ok(None),
    }
}
