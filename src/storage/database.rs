use std::{collections::HashMap, fs, time::{SystemTime, UNIX_EPOCH}};
use actix_web::body::None;
use regex::Regex;
use sqlite::{Connection, Error, State};
use uuid::Uuid;

use crate::{structs::get_outputs::{OutputProtocol, SelectionIdentifier, SelectionIdentifierPair}, TOKEN_VALID_LENGTH};

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
            CREATE TABLE IF NOT EXISTS 'sessions' (id INTEGER not null\nconstraint tokens_pk\nprimary key autoincrement, uuid VARCHAR(36) not null, created INT not null);
            CREATE TABLE IF NOT EXISTS 'admins' (id INTEGER not null\nconstraint admins_pk\nprimary key autoincrement, email TEXT not null);
            CREATE TABLE IF NOT EXISTS 'submissions' (id INTEGER not null\nconstraint submissions_pk\nprimary key autoincrement, uuid VARCHAR(36) not null);
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
                protocol_uuid VARCHAR(36) not null,
                grade INTEGER not null
            );
            "; 


        connection.execute(setup_query).expect("Failed to execute Setup-Instructions!");

        Database {
            connection
        }
    }

    //Authentication

    pub fn save_access_token(&mut self) -> Result<Option<String>, Error> {
        let uuid = match self.get_new_token_uuid() {
            Some(uuid) => uuid,
            None => {
                return Ok(None);
            },
        };
        let query = format!("INSERT INTO sessions(uuid, created) VALUES ('{}', {})", uuid, get_current_time_seconds());
        match self.connection.execute(query) {
            Ok(_) => Ok(Some(uuid)),
            Err(err) => Err(err),
        }
    }

    pub fn remove_expired_sessions(&mut self) -> Result<(), Error> {
        let query = format!("DELETE FROM sessions WHERE created < {};", get_current_time_seconds() - TOKEN_VALID_LENGTH);
        match self.connection.execute(query) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn is_session_valid(&mut self, session_id: &str) -> Result<bool, Error> {
        let query = format!("SELECT uuid FROM sessions WHERE uuid = '{}';", session_id); 
        let mut statement = self.connection.prepare(&query)?;

        if let Ok(State::Row) = statement.next() {
            match statement.read::<String, _>("uuid") {
                Ok(uuid) => {
                    Ok(uuid.eq(session_id))
                },
                Err(err) => Err(err),
            }
        } else {
            Ok(false)
        }
    }

    pub fn check_if_user_admin(&mut self, email: &str) -> Result<bool, Error> {

        if !email_is_safe(email) {
            println!("Got invalid Email!: {:?}", email);
            return Ok(false);
        }

        let query = format!("SELECT email FROM admins WHERE email = '{}';", email); 
        let mut statement = self.connection.prepare(&query)?;

        if let Ok(State::Row) = statement.next() {
            match statement.read::<String, _>("email") {
                Ok(database_email) => {
                    Ok(database_email.eq(&email))
                },
                Err(err) => Err(err),
            }
        } else {
            Ok(false) 
        }
    }

    //Data Manipulation
    
    pub fn add_admin(&mut self, email: &str) -> Result<(), Error> {

        if !email_is_safe(email) {
            println!("Got invalid Email!: {:?}", email);
            return Ok(());
        }

        let query = format!("INSERT INTO admins(email) VALUES ('{}');", email);
        match self.connection.execute(query) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub fn remove_admin(&mut self, email: &str) -> Result<(), Error> {

        if !email_is_safe(email) {
            println!("Got invalid Email!: {:?}", email);
            return Ok(());
        }

        let query = format!("DELETE FROM admins WHERE VALUES ('{}');", email);
        match self.connection.execute(query) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
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

        let validate_regex = Regex::new(r"^([a-zA-Z0-9äöüÄÖÜ]|\.|-|_| )*$").expect("Failed to Assemble Hardcoded Regex!");
        
        if validate_regex.captures(&display_name).is_none() {
            println!("Got unsafe Input: {:?}", display_name);
            return Ok(None);
        }

        let query = format!("SELECT id FROM {} WHERE display_name = '{}';", table_name, display_name);

        let potential_id = match self.if_exists(&query) {
            Ok(exists) => exists,
            Err(err) => {
                return Result::Err(err)
            }
        };

        if let Some(id) = potential_id {
            return Result::Ok(Some(id));
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
            Some(id) => Result::Ok(Some(id)),
            None => Result::Ok(None),
        }

    }


    pub fn save_protocol(&mut self, examiner_subject_relation_ids: Vec<(i64, i64)>, stex_id: i64, season_id: i64, year: i64, protocol: String, grades: Vec<i64>) -> Result<Option<String>, Error> {
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

        for i in 0..examiner_subject_relation_ids.len() {
            let rel = examiner_subject_relation_ids.get(i).expect("Got out of Range exception when in Range");
            let potential_relation_id = match self.create_relation_if_not_exist(rel.0, rel.1, stex_id, season_id, year) {
                Ok(pot) => pot,
                Err(err) => return Result::Err(err),
            };

            let relation_id = match potential_relation_id {
                Some(id) => id,
                None => return Result::Ok(None),
            };

            let corresponding_grade = match grades.get(i) {
                Some(grade) => grade,
                None => return Result::Ok(None),
            };

            match self.connection.execute(format!("INSERT INTO protocols(relation_id, protocol_uuid) VALUES ({}, '{}', {});", relation_id, protocol_uuid, corresponding_grade)) {
                Ok(_) => {},
                Err(err) => return Result::Err(err),
            };
        }

        Result::Ok(Some(protocol_uuid.to_string()))
    }

    pub fn save_submitted_protocol(&mut self) -> Result<Option<String>, Error> {
        let potential_uuid = match self.get_new_submission_uuid() {
            Some(uuid) => uuid,
            None => {
                return Ok(None);
            },
        };

        match self.connection.execute(format!("INSERT INTO submissions(uuid) VALUES ('{}');", potential_uuid)) {
            Ok(_) => {
                Ok(Some(potential_uuid))
            },
            Err(err) => Err(err),
        }
    }

    pub fn remove_submitted_protocol(&mut self, uuid: String) -> Result<bool, Error> {
        if !is_uuid(&uuid) {
            return Ok(false);
        }

        match self.connection.execute(format!("DELETE FROM submissions WHERE uuid = '{}';", uuid)) {
            Ok(_) => Ok(true),
            Err(err) => Err(err),
        }
    }


    //Data Reading

    pub fn search_for_protocol(&self, examiner_ids: Option<Vec<i64>>, subject_ids: Option<Vec<i64>>, stex_ids: Option<Vec<i64>>, seasons: Option<Vec<i64>>, years: Option<Vec<i64>>) -> Result<Option<Vec<OutputProtocol>>, Error> {
        
        let mut search_clause = "".to_string();
        let mut need_and = false;

        need_and = self.build_search_criteria(examiner_ids, &mut search_clause, need_and, "examiner_id");
        need_and = self.build_search_criteria(subject_ids, &mut search_clause, need_and, "subject_id");
        need_and = self.build_search_criteria(stex_ids, &mut search_clause, need_and, "stex_id");
        need_and = self.build_search_criteria(seasons, &mut search_clause, need_and, "season_id");
        let _ = self.build_search_criteria(years, &mut search_clause, need_and, "year");

        let query = format!("
            SELECT protocol_uuid          AS uuid,
                   examiners.display_name AS examiner,
                   subjects.display_name  AS subject,
                   stex.display_name      AS stex,
                   seasons.display_name   AS season,
                   year
            FROM (SELECT protocol_uuid, examiner_id, subject_id, season_id, stex_id, year
                  FROM (SELECT examiner_id, subject_id, season_id, stex_id, year, protocol_uuid
                        FROM subject_relations
                                 JOIN protocols ON subject_relations.id = protocols.relation_id)
                  WHERE {})
                     JOIN examiners ON examiner_id = examiners.id
                     JOIN subjects ON subject_id = subjects.id
                     JOIN stex ON stex_id = stex.id
                     JOIN seasons ON season_id = seasons.id;
        ", search_clause);

        let mut statement = self.connection.prepare(&query)?;

        let mut working_search_results: HashMap<String, OutputProtocol> = HashMap::new();

        while let Ok(State::Row) = statement.next() {

            let uuid = statement.read::<String, _>("uuid")?;
            let examiner = statement.read::<String, _>("examiner")?;
            let subject = statement.read::<String, _>("subject")?;
            let stex = statement.read::<String, _>("stex")?;
            let season = statement.read::<String, _>("season")?;
            let year = statement.read::<i64, _>("year")?;

            match working_search_results.get_mut(&uuid) {
                Some(protocol) => {

                    if !protocol.subject_examiners.contains(&(examiner.clone(), subject.clone())) {
                        protocol.subject_examiners.push((examiner, subject));
                    }

                    if !protocol.stex.contains(&stex) {
                        protocol.stex.push(stex);
                    }

                    if !protocol.season.contains(&season) {
                        protocol.season.push(season);
                    }

                    if !protocol.years.contains(&year) {
                        protocol.years.push(year)
                    }
                },
                None => {
                    working_search_results.insert(uuid.clone(), OutputProtocol { uuid, subject_examiners: vec![(examiner, subject)], stex: vec![stex], season: vec![season], years: vec![year] });
                },
            };
        }

        let mut search_results = vec![];

        for (_, result) in working_search_results {
            search_results.push(result);
        }

        Result::Ok(Some(search_results))
    }

    pub fn remove_protocol(&mut self, uuid: &str) -> Result<bool, Error> {
        if !is_uuid(&uuid) {
            return Ok(false);
        }

        match self.connection.execute(format!("DELETE FROM protocols WHERE protocol_uuid = '{}';", uuid)) {
            Ok(_) => Ok(true),
            Err(err) => Err(err),
        }
    }

    pub fn list_protocols(& self) -> Result<Vec<String>, Error> {
        let query = "SELECT uuid FROM submissions";
        let mut statement = self.connection.prepare(query).expect("Failed to prepare hardcoded Request");

        let mut uuids = vec![];

        while let Ok(State::Row) = statement.next() {
            let uuid = match statement.read::<String, _>("email") {
                Ok(uuid) => uuid,
                Err(err) => return Err(err),
            };
            uuids.push(uuid);
        }

        Ok(uuids)
    }


    pub fn get_selection_identifiers(&self) -> Result<SelectionIdentifier, Error> {
        
        let mut identifiers = SelectionIdentifier { examiners: vec![], subjects: vec![], stex: vec![], seasons: vec![] };

        match self.request_selection_identifiers("examiners" , &mut identifiers.examiners) {
            Ok(_) => {},
            Err(err) => return Err(err),
        };

        match self.request_selection_identifiers("subjects" , &mut identifiers.subjects) {
            Ok(_) => {},
            Err(err) => return Err(err),
        };

        match self.request_selection_identifiers("stex" , &mut identifiers.stex) {
            Ok(_) => {},
            Err(err) => return Err(err),
        };

        match self.request_selection_identifiers("seasons" , &mut identifiers.seasons) {
            Ok(_) => {},
            Err(err) => return Err(err),
        };

        Ok(identifiers)
    }

    pub fn get_admins(&self) -> Result<Vec<String>, Error> {
        let mut statement = match self.connection.prepare("SELECT email FROM admins;") {
            Ok(statement) => statement,
            Err(err) => return Err(err),
        };


        let mut admins = vec![];

        while let Ok(State::Row) = statement.next() {
            let mail = match statement.read::<String, _>("email") {
                Ok(mail) => mail,
                Err(err) => return Err(err),
            };
            admins.push(mail);
        }

        Ok(admins)
    }

    //Helper Methods

    fn request_selection_identifiers(&self, target_table: &str, identifiers: &mut Vec<SelectionIdentifierPair>) -> Result<(), Error> {
        let query = format!("SELECT * FROM {};", target_table);
        let mut statement = match self.connection.prepare(&query) {
            Ok(statement) => statement,
            Err(err) => return Err(err),
        };

        while let Ok(State::Row) = statement.next() {
            let display_name = match statement.read::<String, _>("display_name") {
                Ok(id) => id,
                Err(err) => return Err(err),
            };

            let id = match statement.read::<i64, _>("id") {
                Ok(id) => id,
                Err(err) => return Err(err),
            };

            identifiers.push(SelectionIdentifierPair { id, display_name });
        }

        Ok(())
    }

    #[allow(unused_assignments)]//<- The linter doesn't like what "need_and" does... 
    fn build_search_criteria(&self, input_ids: Option<Vec<i64>>, search_clause: &mut String, mut need_and: bool, search_criteria: &str) -> bool{
        if let Some(ids) = input_ids { 
            if need_and {
                need_and = false;
                search_clause.push_str(" AND ")
            }
            search_clause.push('(');
            for i in 0..ids.len() {
                search_clause.push_str(&format!("{} = {}", search_criteria, &ids[i].to_string()));
                if i + 1 < (ids.len()) {
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
            Some(_) => self.get_new_uuid(),
            None => Some(potential_uuid),
        }
    }

    /// This method returns a UUID that is Unique (in this database)
    /// Potentially it can recourse an infinite amount of times, but thats statistically VERY VERY
    /// VERY UNLIKELY
    fn get_new_token_uuid(&self) -> Option<String> {
        let potential_uuid = Uuid::new_v4().to_string();

        let potential_id = match self.if_exists(&format!("SELECT id FROM sessions WHERE uuid = '{}';", potential_uuid)) {
            Ok(exists) => exists,
            Err(err) => {
                println!("Failed to check if uuid exists: {:?}", err); 
                return None;
            }
        };

        match potential_id {
            Some(_) => self.get_new_uuid(),
            None => Some(potential_uuid),
        }
    }
    
    /// This method returns a UUID that is Unique (in this database)
    /// Potentially it can recourse an infinite amount of times, but thats statistically VERY VERY
    /// VERY UNLIKELY
    fn get_new_submission_uuid(&self) -> Option<String> {
        let potential_uuid = Uuid::new_v4().to_string();

        let potential_id = match self.if_exists(&format!("SELECT id FROM sessions WHERE uuid = '{}';", potential_uuid)) {
            Ok(exists) => exists,
            Err(err) => {
                println!("Failed to check if uuid exists: {:?}", err); 
                return None;
            }
        };

        match potential_id {
            Some(_) => self.get_new_uuid(),
            None => Some(potential_uuid),
        }
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

        if let Some(id) = potential_id {
            return Result::Ok(Some(id))
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
            Some(id) => Result::Ok(Some(id)),
            None => Result::Ok(None),
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

pub fn get_current_time_seconds() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).expect("naja lolm, die Zeit hat sich zurückbewegt...");
    since_the_epoch.as_secs()
}

fn email_is_safe(potentially_unsafe_email: &str) -> bool {
    let regex = Regex::new(r"^(([a-zA-Z]|[0-9]|-|_)*(\.)?)*\+?([a-zA-Z]|[0-9])*@(([a-zA-Z]|[0-9]|-)*(\.)?)*([a-zA-Z]|[0-9])*\.([a-zA-Z]|[0-9])*$").expect("Failed to Construct hardcoded email Regex!");

    regex.captures(potentially_unsafe_email).is_some()
}

fn is_uuid(potentially_unsafe_uuid: &str) -> bool {
    let regex = Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$").expect("Failed to Construct hardcoded UUID Regex");

    regex.captures(potentially_unsafe_uuid).is_some()
}
