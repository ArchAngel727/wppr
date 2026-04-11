use chrono::{DateTime, FixedOffset};

#[derive(PartialOrd, PartialEq, Eq, Debug)]
pub struct Image {
    pub link: String,
    pub date: DateTime<FixedOffset>,
}

impl Image {
    pub fn new() -> Image {
        Image {
            link: String::new(),
            date: DateTime::default(),
        }
    }
}
