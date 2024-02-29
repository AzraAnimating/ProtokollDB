use std::{sync::Arc, fs};

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use storage::database::Database;
use tokio::sync::Mutex;

mod storage;

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
            .service(hello)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


#[get("/")]
async fn hello(data: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
    let mut database = data.lock().await;
    let examiner_id = database.create_examiner("Yuppimen".to_string()).expect("Help").unwrap();
    let subject_id = database.create_subject("essi 2".to_string()).expect("Helpus").unwrap();
    let stex_id = database.create_stex("17.".to_string()).expect("Helpas").unwrap();
    let season_id = database.create_season("Tiefster Winter".to_string()).expect("Heldup").unwrap();
    let protocol_id = database.save_protocol(examiner_id, subject_id, stex_id, season_id, 2024, "yeah HALLO".to_string()).expect("GLUGLUG").unwrap();
    HttpResponse::Ok().body(format!("{}", protocol_id))
}
