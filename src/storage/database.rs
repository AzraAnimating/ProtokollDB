use sqlite::Connection;

pub struct Database {
    connection: Connection
}

pub struct DatabaseConnectionInfo {
    pub hostname: String, 
    pub port: u16, 
    pub username: String, 
    pub password: String, 
    pub database: String
}

impl Database {
    pub fn new(_conn_info: Option<DatabaseConnectionInfo>) -> Database {//ToDo: Actually make this usable
        let connection = sqlite::open("index.db").expect("Failed to connect to local database?!?!!?");
        
        let setup_query = "
            CREATE TABLE IF NOT EXISTS 'examiners' (id INTEGER not null\nconstraint examiners_pk\nprimary key autoincrement, display_name TEXT not null);
            CREATE TABLE IF NOT EXISTS 'subjects' (id INTEGER not null\nconstraint subjects_pk\nprimary key autoincrement, display_name TEXT not null);
            CREATE TABLE IF NOT EXISTS 'stex' (id INTEGER not null\nconstraint stex_pk\nprimary key autoincrement, display_name TEXT not null);
            CREATE TABLE IF NOT EXISTS 'seasons' (id INTEGER not null\nconstraint seasons_pk\nprimary key autoincrement, display_name TEXT not null);
            CREATE TABLE IF NOT EXISTS 'subject_relations' (
                id INTEGER not null\nconstraint subject_relations_pk\nprimary key autoincrement,
                examiner_id INTEGER not null\nconstraint subject_relations_examiners_id_fk\nreferences examiners,
                subject_id INTEGER not null\nconstraint subject_relations_subjects_id_fk\nreferences subjects,
                stex_id INTERGER not null\nconstraint subject_relations_stex_id_fk\nreferences stex,
                season_id INTEGER not null\nconstraint subject_relations_seasons_id_fk\nreferences seasons,
                year INTEGER not null
            );
            CREATE TABLE IF NOT EXISTS 'protocols' (
                id INTEGER not null\nconstraint protocols_pk\nprimary key autoincrement,
                relation_id INTEGER not null\nconstraint protocols_subject_relations_id_fk\nreferences subject_relations,
                protocol_uuid VARCHAR(36) not null
            );
            "; 


        match connection.execute(setup_query) {
            Ok(_) => {},
            Err(err) => panic!("Failed to write Setup Query to internal Database: {:?}", err),
        }

        Database {
            connection
        }
    }
}
