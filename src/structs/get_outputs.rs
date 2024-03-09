use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OutputProtocol {
    pub uuid: String, 
    pub subject_examiners: Vec<(String, String)>, 
    pub stex: Vec<String>, 
    pub season: Vec<String>, 
    pub years: Vec<i64>
}

#[derive(Serialize, Deserialize)]
pub struct SelectionIdentifier {
    pub examiners: Vec<SelectionIdentifierPair>,
    pub subjects: Vec<SelectionIdentifierPair>, 
    pub stex: Vec<SelectionIdentifierPair>,
    pub seasons: Vec<SelectionIdentifierPair>
}

#[derive(Serialize, Deserialize)]
pub struct SelectionIdentifierPair {
    pub id: i64, 
    pub display_name: String
}

#[derive(Serialize)]
pub struct ProtocolList {
    pub protocols: Vec<String>
}
