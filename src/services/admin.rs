use std::{fs, sync::Arc, thread::park_timeout_ms};

use actix_web::{delete, get, http::header::ContentType, post, web::{self, Json}, HttpRequest, HttpResponse, Responder};
use tokio::sync::Mutex;

use crate::{authenticate_admin, expose_error, invalid_input, services::common::authenticate_admin, storage::database::Database, structs::{configuration::{self, Configuration}, get_outputs::ProtocolList, post_inputs::{ChangeAdmin, Create, CreateField, Protocol}}};


#[post("/api/admin/v1/save")]
pub async fn save_protocol(request: HttpRequest, protocol: Json<Protocol>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {

    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

    let mut database = data.lock().await; 
    let potential_protocol_uuid = match database.save_protocol(protocol.examiner_subject_ids.clone(), protocol.stex_id, protocol.season_id, protocol.year, protocol.text.clone(), protocol.grades.clone()) {
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

    if let Some(uuid) = &protocol.submission_id {
        match database.remove_protocol(uuid) {
            Ok(_) => {},
            Err(err) => {
                println!("Failed to remove Protocol Submission '{}'!: {:?}", uuid, err)
            },
        }

        drop(database);

        match fs::remove_file(format!("submitted_protocols/{}.json", uuid)) {
            Ok(_) => {},
            Err(err) => println!("Failed to remove Protocol Submission '{}' from Files!: {:?}", uuid, err),
        }
    }

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

#[get("/api/admin/v1/submissions")]
pub async fn get_submitted_protocols(request: HttpRequest, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {
    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

    let database = data.lock().await;

    let protocols = match database.list_protocols() {
        Ok(protocols) => protocols,
        Err(err) => {
            expose_error!(&format!("Failed to list Protocols!: {:?}", err)); 
        },
    };

    HttpResponse::Ok().content_type(ContentType::json()).json(ProtocolList { protocols })
}

#[post("/api/admin/v1/addadmin")]
pub async fn add_admin(request: HttpRequest, admin: Json<ChangeAdmin>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {
    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

    let mut database = data.lock().await;

    if let Err(err) = database.add_admin(&admin.email_addr) {
        expose_error!(&format!("Failed to add Admin!: {:?}", err));
    };
    
    HttpResponse::Ok().body("")
}


#[delete("/api/admin/v1/removeadmin")]
pub async fn remove_admin(request: HttpRequest, admin: Json<ChangeAdmin>, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {
    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());

    let mut database = data.lock().await;

    if let Err(err) = database.remove_admin(&admin.email_addr) {
        expose_error!(&format!("Failed to remove Admin!: {:?}", err));
    };
    
    HttpResponse::Ok().body("")
}

#[get("/api/admin/v1/getadmins")]
pub async fn list_admins(request: HttpRequest, data: web::Data<Arc<Mutex<Database>>>, configuration: web::Data<Configuration>) -> impl Responder {
    authenticate_admin!(request, data.clone(), configuration.encryption.token_encryption_secret.clone());


    let database = data.lock().await;

    let admins = match database.get_admins() {
        Ok(admins) => admins,
        Err(err) => {
            expose_error!(&format!("Failed to list Admins!: {:?}", err));
        },
    };

    let return_str = format!("{{\"admins\": {:?} }}", admins);
    
    HttpResponse::Ok().content_type(ContentType::json()).body(return_str)
}
