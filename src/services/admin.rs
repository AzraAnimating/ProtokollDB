use std::sync::Arc;

use actix_web::{http::header::ContentType, post, web::{self, Json}, HttpRequest, HttpResponse, Responder};
use tokio::sync::Mutex;

use crate::{authenticate_admin, expose_error, invalid_input, services::common::authenticate_admin, storage::database::Database, structs::{configuration::Configuration, post_inputs::{Create, CreateField, Protocol}}};


#[post("/api/admin/v1/save")]
pub async fn save_protocol(request: HttpRequest, protocol: Json<Protocol>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {

    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

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

#[post("/api/admin/v1/create")]
pub async fn create(request: HttpRequest, creation: Json<Create>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>)  -> impl Responder {
    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

    let mut database = data.lock().await;

    let potential_id = match creation.field {
        CreateField::Examiner => {
            match database.create_examiner(creation.display_name.clone()) {
                Ok(id) => id,
                Err(err) => {
                    expose_error!(&format!("Failed to create Examiner!: {:?}", err));
                },
            }
        },
        CreateField::Subject => {
            match database.create_subject(creation.display_name.clone()) {
                Ok(id) => id,
                Err(err) => {
                    expose_error!(&format!("Failed to create Subject!: {:?}", err));
                },
            }
        },
        CreateField::Season => {
            match database.create_season(creation.display_name.clone()) {
                Ok(id) => id,
                Err(err) => {
                    expose_error!(&format!("Failed to create Season!: {:?}", err));
                },
            }
        },
        CreateField::Stex => {
            match database.create_stex(creation.display_name.clone()) {
                Ok(id) => id,
                Err(err) => {
                    expose_error!(&format!("Failed to create Stex!: {:?}", err));
                },
            }
        },
    };

    drop(database);

    let id = match potential_id {
        Some(id) => id,
        None => {
            expose_error!("No ID created!");
        },
    };

    HttpResponse::Ok().content_type(ContentType::json()).body("{\"created_id\":\"<ID>\"}".replace("<ID>", &id.to_string()))

}
