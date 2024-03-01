#[derive(Serialize, Deserialize)]
pub struct Search {
    pub subjects: Option<Vec<i64>>, 
    pub stex: Option<Vec<i64>>,
    pub examiners: Option<Vec<i64>>,
    pub seasons: Option<Vec<i64>>,
    pub years: Option<Vec<i64>>,
}
