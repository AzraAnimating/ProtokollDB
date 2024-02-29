use std::sync::Arc;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use storage::database::Database;
use tokio::sync::Mutex;

mod storage;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

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
    let res = data.lock().await.create_examiner("michael".to_string()).expect("Help");
    HttpResponse::Ok().body(format!("{}", res.unwrap()))
}
