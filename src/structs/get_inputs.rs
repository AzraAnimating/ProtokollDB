use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Search {
    pub subjects: Option<String>, 
    pub stex: Option<String>,
    pub examiners: Option<String>,
    pub seasons: Option<String>,
    pub years: Option<String>,
}
