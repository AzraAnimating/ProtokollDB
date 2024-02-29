use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Protocol {
    pub examiner_subject_ids: Vec<(i64, i64)>, 
    pub stex_id: i64,
    pub season_id: i64,
    pub year: i64,
    pub text: String
}
