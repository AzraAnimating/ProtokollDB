use std::{fs, num::ParseIntError, sync::Arc};

use actix_web::{get, http::header::ContentType, post, web::{self, Json, Query}, App, HttpResponse, HttpServer, Responder};
use openssl::{pkey::{Id, PKey}, rsa::Rsa};
use storage::database::Database;
use structs::post_inputs::Protocol;
use tokio::sync::Mutex;

use crate::{services::{admin::save_protocol, display::{home, info, invalid_auth}, openidconnect, user::search_for_protocol}, structs::{configuration::{Authorization, Configuration}, get_inputs::Search}};


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

    let write_key_to_file;

    let key_result = match fs::read_to_string(&configuration.encryption.private_key_file) {
        Ok(key) => {
            write_key_to_file = false;
            PKey::private_key_from_pem(key.as_bytes())
        },
        Err(err) => {
            write_key_to_file = true;
            println!("No Private Key present - Generating new one!: {:?}", err);
            PKey::from_rsa(Rsa::generate(2048).expect("Failed to generate RSA!"))
        },
    };

    let key = key_result.expect("Failed to extract Key from result!");

    if write_key_to_file {
        let key_str = key.private_key_to_pem_pkcs8().expect("Failed to Serialize Privatekey!");

        let _ = fs::write(configuration.encryption.private_key_file.clone(), key_str);
    }

    let key_ptr = Arc::new(key);

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
    let movable_key_ptr = key_ptr.clone();

    HttpServer::new(move || {
        let mov_config = movable_config.clone();
        let mov_key_ptr = movable_key_ptr.clone();
        let app = App::new()
            .app_data(web::Data::new(Arc::new(Mutex::new(Database::new(None)))))
            .app_data(web::Data::new(mov_config))
            .app_data(web::Data::new(mov_key_ptr))
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
