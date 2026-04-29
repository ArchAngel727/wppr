use std::fmt::Display;

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

impl Display for OnlineImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.link)
    }
}
