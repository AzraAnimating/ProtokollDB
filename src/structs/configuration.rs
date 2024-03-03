use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub database_type: DatabaseBackend,
    pub api: APISettings,
    pub authorization: Authorization,
    pub encryption: Encryption,
    pub general: Generals,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, Clone)]
pub struct APISettings {
    pub bind_addr: String, 
    pub bind_port: u16,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Generals {
    pub protocol_location: String
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Authorization {
    OpenIdConnect {
        client_id: String, 
        self_root_url: String,
        token_url: String, 
        auth_url: String, 
        revoke_url: String, 
        userinfo_url: String
    },
    None
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Encryption {
    pub token_encryption_secret: String
}

impl Configuration {
    pub fn default() -> Configuration {
        Configuration {
            database_type: DatabaseBackend::SQLLite { file_location: "index.db".to_string() },
            api: APISettings { bind_addr: "127.0.0.1".to_string(), bind_port: 8080 },
            authorization: Authorization::OpenIdConnect { token_url: "plz".to_owned(), auth_url: "replace".to_string(), revoke_url: "to".to_string(), userinfo_url: "actual_urls".to_string(), client_id: "yikksi".to_string(), self_root_url: "http://127.0.0.1".to_string() },
            general: Generals { protocol_location: "protocols/".to_string() },
            encryption: Encryption { token_encryption_secret: thread_rng().sample_iter(&Alphanumeric).take(10).map(char::from).collect() },
        }
    }
}
