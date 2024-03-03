use std::{num::ParseIntError, sync::Arc};

use actix_web::{get, http::header::ContentType, web::{self, Query}, HttpRequest, HttpResponse, Responder};
use tokio::sync::Mutex;

use crate::{authenticate, expose_error, invalid_input, storage::database::Database, structs::{configuration::Configuration, get_inputs::Search}};

use super::common::authenticate;


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
