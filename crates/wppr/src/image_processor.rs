use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use image::{DynamicImage, ImageReader};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::path::PathBuf;
use tokio::{fs, sync::mpsc};

use crate::local_image::LocalImage;

pub struct ImageProcessor {
    pub rx: mpsc::Receiver<(Result<StatefulProtocol>, LocalImage)>,
}

#[derive(Clone)]
pub struct ImageProcessorArgs {
    path: Option<PathBuf>,
    local_images: Option<Vec<LocalImage>>,
}

impl ImageProcessorArgs {
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            path: Some(path),
            local_images: None,
        }
    }

    pub fn from_local_images(vec: Vec<LocalImage>) -> Self {
        Self {
            path: None,
            local_images: Some(vec),
        }
    }
}

impl ImageProcessor {
    pub fn new(picker: Picker, args: ImageProcessorArgs) -> Self {
        let local_image_rx = Self::load_images_from_fs(args);
        let dynamic_image_rx = Self::load_images_from_file(local_image_rx);
        let protocol_rx = Self::create_protocol_from_image(dynamic_image_rx, picker);

        Self { rx: protocol_rx }
    }

    fn load_images_from_fs(args: ImageProcessorArgs) -> mpsc::Receiver<LocalImage> {
        let (tx, rx) = mpsc::channel::<LocalImage>(16);

        tokio::spawn(async move {
            if let Some(local_images) = args.local_images {
                for local_image in local_images {
                    if tx.send(local_image).await.is_err() {
                        break;
                    }
                }
            }

            let Some(path) = args.path else {
                return;
            };

            let mut entries = match fs::read_dir(&path).await {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("read_dir {}: {}", path.display(), e);
                    return;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let metadata = match entry.metadata().await {
                    Ok(metadata) => metadata,
                    _ => continue,
                };

                let date: DateTime<Utc> = metadata
                    .modified()
                    .ok()
                    .map(DateTime::from)
                    .unwrap_or(Utc::now());

                let local_image = LocalImage::from((entry.path(), date));

                if tx.send(local_image).await.is_err() {
                    break;
                }
            }
        });

        rx
    }

    fn load_images_from_file(
        mut image_tx: mpsc::Receiver<LocalImage>,
    ) -> mpsc::Receiver<(Result<DynamicImage>, LocalImage)> {
        let (tx, rx) = mpsc::channel::<(Result<DynamicImage>, LocalImage)>(16);

        tokio::spawn(async move {
            while let Some(image) = image_tx.recv().await {
                let tx = tx.clone();

                tokio::task::spawn_blocking(move || {
                    let result = (|| -> Result<DynamicImage> {
                        ImageReader::open(&image.path)?
                            .with_guessed_format()?
                            .decode()
                            .map_err(|e| anyhow!("decode {}: {}", image.path.display(), e))
                    })();

                    let _ = tx.blocking_send((result, image));
                });
            }
        });

        rx
    }

    fn create_protocol_from_image(
        mut image_tx: mpsc::Receiver<(Result<DynamicImage>, LocalImage)>,
        picker: Picker,
    ) -> mpsc::Receiver<(Result<StatefulProtocol>, LocalImage)> {
        let (tx, rx) = mpsc::channel::<(Result<StatefulProtocol>, LocalImage)>(16);

        tokio::spawn(async move {
            while let Some(image) = image_tx.recv().await {
                let tx = tx.clone();
                let picker = picker.clone();

                tokio::task::spawn_blocking(move || {
                    let result = (|| -> Result<StatefulProtocol> {
                        let image = image.0?;

                        Ok(picker.new_resize_protocol(image))
                    })();

                    let _ = tx.blocking_send((result, image.1));
                });
            }
        });

        rx
    }
}
