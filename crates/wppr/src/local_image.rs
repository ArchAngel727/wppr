use chrono::{DateTime, FixedOffset};
use std::{fmt, path::PathBuf};

#[derive(PartialOrd, PartialEq, Eq)]
pub struct LocalImage {
    pub path: PathBuf,
    pub date: DateTime<FixedOffset>,
}

impl From<(PathBuf, DateTime<FixedOffset>)> for LocalImage {
    fn from((path, date): (PathBuf, DateTime<FixedOffset>)) -> Self {
        Self { path, date }
    }
}

impl fmt::Display for LocalImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path: {}, Date: {}", self.path.display(), self.date)
    }
}
