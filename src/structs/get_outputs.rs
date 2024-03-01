use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OutputProtocol {
    pub uuid: String, 
    pub examiners: Vec<String>, 
    pub subjects: Vec<String>, 
    pub stex: Vec<String>, 
    pub season: Vec<String>, 
    pub years: Vec<i64>
}
