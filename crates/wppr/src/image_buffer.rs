use chrono::{DateTime, Utc};
use ratatui_image::protocol::StatefulProtocol;

pub struct ImageBuffer {
    pub protocols: Vec<StatefulProtocol>,
    pub timestamps: Vec<DateTime<Utc>>,
}

impl ImageBuffer {
    pub fn new() -> Self {
        Self {
            protocols: Vec::new(),
            timestamps: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        if self.protocols.len() == self.timestamps.len() {
            self.protocols.len()
        } else {
            0
        }
    }

    pub fn sort_by_timestamp(&mut self) {
        let mut pairs: Vec<_> = self
            .protocols
            .drain(..)
            .zip(self.timestamps.drain(..))
            .collect();

        pairs.sort_by_key(|(_, timestamp)| *timestamp);

        for (protocol, timestamp) in pairs {
            self.protocols.push(protocol);
            self.timestamps.push(timestamp);
        }
    }
}
