use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Protocol {
    pub examiner_subject_ids: Vec<(i64, i64)>, 
    pub stex_id: i64,
    pub season_id: i64,
    pub year: i64,
    pub text: String
}

#[derive(Serialize, Deserialize)]
pub struct Create {
    pub field:CreateField ,
    pub display_name: String
}

#[derive(Serialize, Deserialize)]
pub enum CreateField {
    Examiner, 
    Subject, 
    Season, 
    Stex
}

#[derive(Serialize, Deserialize)]
pub struct ChangeAdmin {
    pub email_addr: String
}
