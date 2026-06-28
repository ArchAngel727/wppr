use std::path::{Path, PathBuf};

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Connection, SqliteConnection, sqlite::SqliteConnectOptions};
use tracing::error;

use crate::local_image::LocalImage;

pub struct DBManager {}

struct LocalImageDBRow {
    name: String,
    timestamp: DateTime<Utc>,
}

impl DBManager {
    pub async fn get_db_connection() -> Result<SqliteConnection> {
        let db_path = if cfg!(debug_assertions) {
            PathBuf::from("./data/db.sqlite")
        } else {
            dirs::data_dir()
                .expect("Could not find data dir")
                .join("wppr/db.sqlite")
        };

        let opts = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        std::fs::create_dir_all(db_path.parent().unwrap())?;

        Ok(SqliteConnection::connect_with(&opts).await?)
    }

    pub async fn write_local_images_to_db(slice: &[LocalImage]) -> Result<()> {
        let mut conn = DBManager::get_db_connection().await?;

        for img in slice {
            let name = if let Some(name) = img.path.file_name() {
                match name.to_str() {
                    Some(str) => str,
                    None => continue,
                }
            } else {
                continue;
            };

            // TODO: crash or continue
            sqlx::query!(
                "INSERT INTO local_images (name, timestamp) values (?1, ?2)",
                name,
                &img.date
            )
            .execute(&mut conn)
            .await
            .inspect_err(|e| error!("{} {}", e, img))?;
        }

        Ok(())
    }

    pub async fn read_local_images_from_db(wppr_dir: &Path) -> Result<Vec<LocalImage>> {
        let mut conn = DBManager::get_db_connection().await?;
        let vec: Vec<LocalImageDBRow> = sqlx::query_as!(
            LocalImageDBRow,
            r#"SELECT name, timestamp as "timestamp: DateTime<Utc>" FROM local_images"#
        )
        .fetch_all(&mut conn)
        .await
        .inspect_err(|e| error!("{}", e))?
        .into_iter()
        .collect();

        Ok(vec
            .iter()
            .map(|img| {
                let mut path: PathBuf = wppr_dir.into();
                path.push(&img.name);
                path.set_extension("png");

                LocalImage::from((path, img.timestamp))
            })
            .collect())
    }
}
