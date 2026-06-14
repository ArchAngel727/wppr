use ratatui_image::protocol::StatefulProtocol;

use crate::local_image::LocalImage;

pub struct ImageBuffer {
    pub protocols: Vec<StatefulProtocol>,
    pub local_images: Vec<LocalImage>,
}

impl ImageBuffer {
    pub const fn new() -> Self {
        Self {
            protocols: Vec::new(),
            local_images: Vec::new(),
        }
    }

    pub const fn len(&self) -> usize {
        if self.protocols.len() == self.local_images.len() {
            self.protocols.len()
        } else {
            0
        }
    }

    pub fn sort_by_timestamp(&mut self) {
        let mut pair: Vec<_> = self
            .protocols
            .drain(..)
            .zip(self.local_images.drain(..))
            .collect();

        pair.sort_by_key(|(_, local_image)| local_image.date);

        for (protocol, local_image) in pair {
            self.protocols.push(protocol);
            self.local_images.push(local_image);
        }
    }
}
