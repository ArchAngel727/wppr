use chrono::{DateTime, FixedOffset};

#[derive(PartialOrd, PartialEq, Eq)]
pub struct OnlineImage {
    pub link: String,
    pub date: DateTime<FixedOffset>,
}

impl OnlineImage {
    pub fn new() -> Self {
        Self {
            link: String::new(),
            date: DateTime::default(),
        }
    }
}
