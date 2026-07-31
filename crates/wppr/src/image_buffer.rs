use ratatui_image::protocol::StatefulProtocol;

use crate::local_image::LocalImage;

pub struct ImageBuffer {
    pub pair_vec: Vec<(LocalImage, Option<StatefulProtocol>)>,
}

impl ImageBuffer {
    pub const fn new() -> Self {
        Self {
            pair_vec: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub const fn len(&self) -> usize {
        self.pair_vec.len()
    }

    pub fn push(&mut self, pair: (LocalImage, Option<StatefulProtocol>)) {
        self.pair_vec.push(pair);
    }

    pub fn sort_by_timestamp(&mut self) {
        self.pair_vec
            .sort_by_key(|(local_image, _)| local_image.date);
        self.pair_vec.reverse();
    }

    #[allow(dead_code)]
    pub fn sort_by_name(&mut self) {
        self.pair_vec
            .sort_by(|(local_image_1, _), (local_image_2, _)| {
                let str1 = local_image_1.path.to_str().unwrap_or("");
                let str2 = local_image_2.path.to_str().unwrap_or("");

                str1.cmp(str2)
            });
    }
}
