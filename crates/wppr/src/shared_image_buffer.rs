use ratatui_image::protocol::StatefulProtocol;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::Notify;

use crate::{image_buffer::ImageBuffer, local_image::LocalImage};

#[derive(Clone)]
pub struct SharedImageBuffer {
    buffer: Arc<Mutex<ImageBuffer>>,
    pub notify: Arc<Notify>,
}

impl SharedImageBuffer {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(ImageBuffer::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn push_pair(&self, pair: (LocalImage, Option<StatefulProtocol>)) {
        self.with(|buffer| {
            if let Some(protocol) = pair.1 {
                if let Some((_, vec_protocol)) = buffer
                    .pair_vec
                    .iter_mut()
                    .find(|(local_image, _)| *local_image == pair.0)
                {
                    *vec_protocol = Some(protocol);
                }
            } else {
                buffer.push(pair);
                buffer.sort_by_timestamp();
            }
        });

        self.notify.notify_one();
    }

    pub fn with<R>(&self, f: impl FnOnce(&mut MutexGuard<ImageBuffer>) -> R) -> R {
        let mut guard = self.buffer.lock().unwrap();
        f(&mut guard)
    }

    pub fn with_slice<R>(
        &self,
        range: std::ops::Range<usize>,
        f: impl FnOnce(&mut [(LocalImage, Option<StatefulProtocol>)]) -> R,
    ) -> R {
        let mut guard = self.buffer.lock().unwrap();
        f(&mut guard.pair_vec[range])
    }

    pub async fn wait_for_update(&self) {
        self.notify.notified().await;
    }
}
