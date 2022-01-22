use chrono::naive::NaiveDateTime;

#[derive(Debug, PartialEq)]
pub struct FileInfo {
    pub hash: String,
    pub date: Option<NaiveDateTime>,
    pub thumb: Vec<u8>,
}
