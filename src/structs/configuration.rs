use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub database_type: DatabaseBackend,
    pub api: APISettings,
    pub general: Generals
}

#[derive(Serialize, Deserialize)]
pub enum DatabaseBackend {
    SQLLite {
        file_location: String
    },
    PostgeSQL {
        hostname: String, 
        port: u16, 
        username: String, 
        password: String, 
        database: String
    }
}

#[derive(Serialize, Deserialize)]
pub struct APISettings {
    pub bind_addr: String, 
    pub bind_port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct Generals {
    pub protocol_location: String
}
