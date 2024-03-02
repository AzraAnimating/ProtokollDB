use std::sync::Arc;

use actix_web::{http::header::ContentType, post, web::{self, Json}, HttpResponse, Responder};
use tokio::sync::Mutex;

use crate::{expose_error, storage::database::Database, structs::post_inputs::Protocol};


#[post("/api/admin/v1/save")]
pub async fn save_protocol(protocol: Json<Protocol>, data: web::Data<Arc<Mutex<Database>>>) -> impl Responder {
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
