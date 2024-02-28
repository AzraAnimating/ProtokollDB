use sqlite::Connection;

pub struct Database {
    connection: Connection
}

impl Database {
    pub fn new() -> Database {
        let connection = match sqlite::open("index.db") {
            Ok(conn) => conn,
            Err(err) => {
                panic!("Failed to connect to internal Database?!?! {:?}", err);
            },
        };
        
        let setup_query = "
            CREATE TABLE IF NOT EXISTS 'examiners' (id INTEGER not null\nconstraint examiners_key\nprimary key autoincrement, display_name TEXT not null);
            ";//ToDo Create the other stuff :) 


        match connection.execute(setup_query) {
            Ok(_) => {},
            Err(err) => panic!("Failed to write Setup Query to internal Database: {:?}", err),
        }

        Database {
            connection
        }
    }
}
