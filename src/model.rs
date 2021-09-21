use chrono::naive::NaiveDateTime;

pub struct FileInfo {
    pub hash: String,
    pub date: Option<NaiveDateTime>,
    pub thumb: Vec<u8>,
}

