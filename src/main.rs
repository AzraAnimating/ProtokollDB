use std::{sync::Arc, fs};

use actix_web::{get, web::{self, Json}, App, HttpResponse, HttpServer, Responder, post, http::header::ContentType};
use storage::database::Database;
use structs::post_inputs::Protocol;
use tokio::sync::Mutex;


mod storage;
mod structs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

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

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(Arc::new(Mutex::new(Database::new(None)))))
            .service(home)
            .service(info)
            .service(save_protocol)
    })
    .bind(("127.0.0.1", 8080))?
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

#[macro_export]
macro_rules! expose_error {
    ($err:expr) => {
        return HttpResponse::InternalServerError().content_type(ContentType::json()).body("{\"error\":\"<E>\"}".to_string().replace("<E>", $err))
    };
}
