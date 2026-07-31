use anyhow::{Result, anyhow};
use image::{DynamicImage, ImageReader};
use ratatui_image::picker::Picker;
use tokio::sync::mpsc::{self, error::TrySendError};
use tracing::error;

use crate::{image_buffer::SharedImageBuffer, local_image::LocalImage};

pub struct ImageProcessor {
    buffer: SharedImageBuffer,
    tx: mpsc::Sender<LocalImage>,
}

impl ImageProcessor {
    pub fn new(picker: Picker) -> Self {
        let shared_buffer = SharedImageBuffer::new();
        let buf = shared_buffer.clone();

        let (local_image_tx, local_image_rx) = mpsc::channel::<LocalImage>(16);
        let dynamic_image_rx = Self::load_images_from_file(local_image_rx);
        Self::create_protocol_from_image(dynamic_image_rx, picker, buf);

        Self {
            buffer: shared_buffer,
            tx: local_image_tx,
        }
    }

    pub fn process(&self, image: LocalImage) {
        match self.tx.try_send(image) {
            Ok(()) => {}
            Err(TrySendError::Full(_img)) => {}
            Err(TrySendError::Closed(_)) => error!("send image channel closed"),
        }
    }

    pub fn get_shared_buffer(&self) -> &SharedImageBuffer {
        &self.buffer
    }

    pub async fn push_image(&self, local_image: LocalImage) {
        self.buffer.push_pair((local_image, None));
    }

    pub async fn wait_for_update(&self) {
        self.buffer.wait_for_update().await;
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
        buffer: SharedImageBuffer,
    ) {
        tokio::spawn(async move {
            while let Some(image) = image_tx.recv().await {
                let picker = picker.clone();
                let buffer = buffer.clone();

                tokio::task::spawn(async move {
                    let (dyn_image, local_image) = image;
                    let dyn_image = dyn_image.unwrap();
                    let protocol = picker.new_resize_protocol(dyn_image);

                    buffer.push_pair((local_image, Some(protocol)));
                });
            }
        });
    }
}
