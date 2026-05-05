use chrono::{DateTime, Utc};
use std::{
    fmt,
    path::{Path, PathBuf},
};

#[derive(PartialOrd, PartialEq, Eq, Clone, Debug)]
pub struct LocalImage {
    pub path: PathBuf,
    pub date: DateTime<Utc>,
}

impl From<(PathBuf, DateTime<Utc>)> for LocalImage {
    fn from((path, date): (PathBuf, DateTime<Utc>)) -> Self {
        Self { path, date }
    }
}

impl fmt::Display for LocalImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Path: {}, Date: {}", self.path.display(), self.date)
    }
}

impl AsRef<Path> for LocalImage {
    fn as_ref(&self) -> &Path {
        self.path.as_ref()
    }
}
