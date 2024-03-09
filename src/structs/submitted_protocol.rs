use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SubmittedProtocol {
    pub author: String, 
    pub subject_examiners: Vec<(i64, i64)>, 
    pub grades: Vec<i64>, 
    pub stex: i64, 
    pub year: i64, 
    pub season: i64, 
    pub hand_in_date: u64,
}
