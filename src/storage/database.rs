use std::fs;

use sqlite::{Connection, Error, State};
use uuid::Uuid;

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
            Some(id) => return Result::Ok(Some(id)),
            None => return Result::Ok(None),
        }

    }


    pub fn save_protocol(&mut self, examiner_subject_relation_ids: Vec<(i64, i64)>, stex_id: i64, season_id: i64, year: i64, protocol: String) -> Result<Option<String>, Error> {
        let protocol_uuid = match self.get_new_uuid() {
            Some(uuid) => uuid,
            None => return Result::Ok(None),
        };
        
        match fs::write(format!("protocols/{}.txt", protocol_uuid), protocol) {
            Ok(_) => {},
            Err(err) => {
                println!("Failed to write protocol to Disk!: {:?}", err); 
                return Result::Ok(None);
            },
        };


        for rel in examiner_subject_relation_ids {
            let potential_relation_id = match self.create_relation_if_not_exist(rel.0, rel.1, stex_id, season_id, year) {
                Ok(pot) => pot,
                Err(err) => return Result::Err(err),
            };

            let relation_id = match potential_relation_id {
                Some(id) => id,
                None => return Result::Ok(None),
            };

            match self.connection.execute(format!("INSERT INTO protocols(relation_id, protocol_uuid) VALUES ({}, '{}')", relation_id, protocol_uuid)) {
                Ok(_) => {},
                Err(err) => return Result::Err(err),
            };
        }

        Result::Ok(Some(protocol_uuid.to_string()))
    }

    pub fn search_for_protocol(&self, examiner_ids: Option<Vec<i64>>, subject_ids: Option<Vec<i64>>, stex_ids: Option<Vec<i64>>, years: Option<Vec<i64>>) -> Result<Option<String>, Error> {
        
        let mut search_clause = "".to_string();
        let mut need_and = false;

        need_and = self.build_search_criteria(examiner_ids, &mut search_clause, need_and);
        need_and = self.build_search_criteria(subject_ids, &mut search_clause, need_and);
        need_and = self.build_search_criteria(stex_ids, &mut search_clause, need_and);
        need_and = self.build_search_criteria(years, &mut search_clause, need_and);

        let mut query = format!("SELECT DISTINCT protocol_uuid FROM (
            SELECT examiner_id, subject_id, season_id, stex_id, year, protocol_uuid FROM subject_relations JOIN protocols ON subject_relations.id = protocols.relation_id
        ) WHERE {};", search_clause);

        println!("{}", query);

        Result::Ok(None)
    }

    fn build_search_criteria(&self, input_ids: Option<Vec<i64>>, search_clause: &mut String, mut need_and: bool) -> bool{
        if let Some(ids) = input_ids { 
            if need_and {
                need_and = false;
                search_clause.push_str(" AND ")
            }
            search_clause.push('(');
            for i in 0..ids.len() {
                search_clause.push_str(&ids[i].to_string());
                if i + 1 < (ids.len() - 1) {
                    search_clause.push_str(" OR ")
                }
            }
            search_clause.push(')');
            need_and = true;
        } else {
            need_and = false;
        }

        need_and
    }

    /// This method returns a UUID that is Unique (in this database)
    /// Potentially it can recourse an infinite amount of times, but thats statistically VERY VERY
    /// VERY UNLIKELY
    fn get_new_uuid(&self) -> Option<String> {
        let potential_uuid = Uuid::new_v4().to_string();

        let potential_id = match self.if_exists(&format!("SELECT id FROM protocols WHERE protocol_uuid = '{}';", potential_uuid)) {
            Ok(exists) => exists,
            Err(err) => {
                println!("Failed to check if uuid exists: {:?}", err); 
                return None;
            }
        };

        match potential_id {
            Some(_) => return self.get_new_uuid(),
            None => return Some(potential_uuid),
        };
    }
    

    ///This method Creates a Relation in the Relation-Table if it doesn't exist
    ///If it does, it just returns that relation
    fn create_relation_if_not_exist(&mut self, examiner_id: i64, subject_id: i64, stex_id: i64, season_id: i64, year: i64) -> Result<Option<i64>, Error> {
        let query = format!("SELECT id FROM subject_relations WHERE examiner_id = {} AND subject_id = {} AND stex_id = {} AND season_id = {} AND year = {};", examiner_id, subject_id, stex_id, season_id, year);
    
        let potential_id = match self.if_exists(&query) {
            Ok(exists) => exists,
            Err(err) => {
                return Result::Err(err)
            }
        };

        match potential_id {
            Some(id) => return Result::Ok(Some(id)),
            None => {},
        }

        match self.connection.execute(format!("INSERT INTO subject_relations(examiner_id, subject_id, stex_id, season_id, year) VALUES ({}, {}, {}, {}, {});", examiner_id, subject_id, stex_id, season_id, year)) {
            Ok(_) => {},
            Err(err) => return Result::Err(err),
        }

        let potential_id = match self.if_exists(&query) {
            Ok(exists) => exists,
            Err(err) => {
                return Result::Err(err)
            }
        };

        match potential_id {
            Some(id) => return Result::Ok(Some(id)),
            None => return Result::Ok(None),
        }

    }

    fn if_exists(&self, query: &str) -> Result<Option<i64>, Error> {
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

