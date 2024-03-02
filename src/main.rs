use std::{fs, num::ParseIntError, rc::Rc, sync::Arc};

use actix_web::{get, http::header::ContentType, post, services, web::{self, Json, Query}, App, HttpResponse, HttpServer, Responder};
use storage::database::Database;
use structs::post_inputs::Protocol;
use tokio::sync::Mutex;

use crate::{services::openidconnect, structs::{configuration::{APISettings, Authorization, Configuration, DatabaseBackend, Generals}, get_inputs::Search}};


mod storage;
mod structs;
mod services;


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let config_str = match fs::read_to_string("config.toml") {
        Ok(file) => file,
        Err(err) => {
            println!("please Populate the config.toml config!");
            let config_default = toml::to_string(&Configuration::default()).expect("Failed to Serialize Default Configuration!");
            fs::write("config.toml", config_default).expect("Failed to write config file!");
            return Result::Err(err)
        },
    };

    let configuration = toml::from_str::<Configuration>(&config_str).expect("Failed to deserialize Configuration.");

    let _ = fs::create_dir_all("protocols/");

    println!("
     __  __     ______     __     ______     ______     ______     ______    
    /\\ \\_\\ \\   /\\  __ \\   /\\ \\   /\\  ___\\   /\\  __ \\   /\\  == \\   /\\  ___\\   
    \\ \\  __ \\  \\ \\  __ \\  \\ \\ \\  \\ \\ \\____  \\ \\ \\/\\ \\  \\ \\  __<   \\ \\  __\\   
     \\ \\_\\ \\_\\  \\ \\_\\ \\_\\  \\ \\_\\  \\ \\_____\\  \\ \\_____\\  \\ \\_\\ \\_\\  \\ \\_____\\ 
      \\/_/\\/_/   \\/_/\\/_/   \\/_/   \\/_____/   \\/_____/   \\/_/ /_/   \\/_____/ 
                                                                             \n
    H[AI]CORE : FSMED ProtocolDB (Backend) v0.0.1
    Author: Tobias Rempe <tobias.rempe@rub.de>
    Current Maintainer: Tobias Rempe <tobias.rempe@rub.de>");


    let movable_config = configuration.clone();//ToDo: Make this less strange...

    HttpServer::new(move || {
        let mov_config = movable_config.clone();
        let app = App::new()
            .app_data(web::Data::new(Arc::new(Mutex::new(Database::new(None)))))
            .app_data(web::Data::new(mov_config))
            .service(invalid_auth)
            .service(home)
            .service(info)
            .service(save_protocol)
            .service(search_for_protocol);


        match movable_config.authorization {
            Authorization::OpenIdConnect { .. } => {
                app
                    .service(openidconnect::login)
                    .service(openidconnect::redirect)
                    .service(openidconnect::finish)
            },
            Authorization::None => {
                app
            },
        }
    })
    .bind((configuration.api.bind_addr, configuration.api.bind_port))?
    .run()
    .await
}


#[get("/info")]
async fn info() -> impl Responder {
    let banner = "
     __  __     ______     __     ______     ______     ______     ______    
    /\\ \\_\\ \\   /\\  __ \\   /\\ \\   /\\  ___\\   /\\  __ \\   /\\  == \\   /\\  ___\\   
    \\ \\  __ \\  \\ \\  __ \\  \\ \\ \\  \\ \\ \\____  \\ \\ \\/\\ \\  \\ \\  __<   \\ \\  __\\   
     \\ \\_\\ \\_\\  \\ \\_\\ \\_\\  \\ \\_\\  \\ \\_____\\  \\ \\_____\\  \\ \\_\\ \\_\\  \\ \\_____\\ 
      \\/_/\\/_/   \\/_/\\/_/   \\/_/   \\/_____/   \\/_____/   \\/_/ /_/   \\/_____/ 
                                                                             \n
    H[AI]CORE : FSMED ProtocolDB (Backend) v0.0.1
    Author: Tobias Rempe <tobias.rempe@rub.de>
    Current Maintainer: Tobias Rempe <tobias.rempe@rub.de>";
    

    HttpResponse::Ok().body(banner)
}

#[get("/")]
async fn home() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body("
        <html>
            <h1>Fachschaft Medizin</h1>
            <h2>Protokolldatenbank v0.0.1</h2>
            <p>Willkommen auf dem Backend der ProtokollDB. Wenn du etwas mit dieser API entwickeln willst, laden wir dich ein <a href = \"https://docs.fsi.rub.de/s/fsmed-protokolldb-docs\">hier</a> vorbeizuschauen.</p>
            <p>Wenn du auf der Suche nach der eigentlichen Website bist, dann klicke bitte <a href = \"https://leckere.aprikosenmarmela.de\">hier</a>.</p>
        </html>
    ")
}

#[post("/api/admin/v1/save")]
async fn save_protocol(protocol: Json<Protocol>, data: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let mut database = data.lock().await; 
    let potential_protocol_uuid = match database.save_protocol(protocol.examiner_subject_ids.clone(), protocol.stex_id, protocol.season_id, protocol.year, protocol.text.clone()) {
        Ok(pot_id) => pot_id,
        Err(err) => {
            expose_error!(&err.to_string());
        },
    };

    let protocol_uuid = match potential_protocol_uuid {
        Some(id) => id,
        None => {
            expose_error!("No Protocol Saved.");
        },
    };

    HttpResponse::Ok().content_type(ContentType::json()).body("{\"protocol_uuid\":\"<ID>\"}".replace("<ID>", &protocol_uuid))
}

#[get("/api/v1/search")]
async fn search_for_protocol(search_terms: Query<Search>, data: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    
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

#[get("/invalidauth")]
async fn invalid_auth() -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::html()).body("<html><h1>Authentication isn't configured correctly. Please contact your respective Server-Admin</h1></html>'")
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


#[macro_export]
macro_rules! expose_error {
    ($err:expr) => {
        return HttpResponse::InternalServerError().content_type(ContentType::json()).body("{\"error\":\"<E>\"}".to_string().replace("<E>", $err))
    };
}


#[macro_export]
macro_rules! invalid_input {
    ($err:expr) => {
        return HttpResponse::InternalServerError().content_type(ContentType::json()).body("{\"error\":\"<E>\"}".to_string().replace("<E>", $err))
    };
}
