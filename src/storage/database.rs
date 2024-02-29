use std::{io::Read, arch::x86_64::_andn_u32};

use sqlite::{Connection, Error, State};

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


        connection.execute(setup_query).expect("Failed to execute Setup-Instructions!");

        Database {
            connection
        }
    }

    pub fn create_examiner(&mut self, display_name: String) -> Result<Option<i64>, Error> {
        self.create_item("examiners".to_string(), display_name)
    }

    pub fn create_subject(&mut self, display_name: String) -> Result<Option<i64>, Error> {
        self.create_item("subjects".to_string(), display_name)
    }

    pub fn create_stex(&mut self, display_name: String) -> Result<Option<i64>, Error> {
        self.create_item("stex".to_string(), display_name)
    }

    pub fn create_season(&mut self, display_name: String) -> Result<Option<i64>, Error> {
        self.create_item("seasons".to_string(), display_name)
    }

    fn create_item(&mut self, table_name: String, display_name: String) -> Result<Option<i64>, Error> {

        let query = format!("SELECT id FROM {} WHERE display_name = '{}';", table_name, display_name);

        let potential_id = match self.if_exists(&query) {
            Ok(exists) => exists,
            Err(err) => {
                return Result::Err(err)
            }
        };

        match potential_id {
            Some(id) => {
                return Result::Ok(Some(id));
            },
            None => {},
        }


        match self.connection.execute(format!("INSERT INTO {}(display_name) VALUES ('{}');", table_name, display_name)) {
            Ok(_) => {},
            Err(err) => return Result::Err(err),
        };


        let potential_id = match self.if_exists(&query) {
            Ok(exists) => exists,
            Err(err) => {
                return Result::Err(err)
            }
        };

        match potential_id {
            Some(id) => {
                return Result::Ok(Some(id));
            },
            None => {
                return Result::Ok(None);
            },
        }

    }

    

    fn if_exists(&mut self, query: &str) -> Result<Option<i64>, Error> {
        let mut statement = match self.connection.prepare(query) {
            Ok(statement) => statement,
            Err(err) => {
                return Result::Err(err);
            },
        };

        if let Ok(State::Row) = statement.next() {
            Result::Ok(
                match statement.read::<i64, _>("id") {
                    Ok(id) => Some(id),
                    Err(err) => {
                        return Result::Err(err)
                    },
                }
            )
        } else {
            Result::Ok(None)
        }
    }
}

