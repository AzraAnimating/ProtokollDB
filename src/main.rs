use std::{fs, sync::Arc};

use actix_web::{web::{self}, App, HttpServer};
use storage::database::Database;
use tokio::sync::Mutex;

use crate::{services::{admin::save_protocol, display::{home, info, invalid_auth}, openidconnect, user::search_for_protocol}, structs::configuration::{Authorization, Configuration}};


mod storage;
mod structs;
mod services;

pub const TOKEN_VALID_LENGTH: u64 = 86400;


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

    println!("\n\nStarting API!\n");

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
