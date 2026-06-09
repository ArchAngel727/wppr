use std::path::PathBuf;

use crate::local_image::LocalImage;

pub struct Packet {
    pub path: Option<PathBuf>,
    pub local_images: Option<Vec<LocalImage>>,
}

impl Packet {
    pub fn from_path(path: PathBuf) -> Self {
        Packet {
            path: Some(path),
            local_images: None,
        }
    }

    pub fn from_img_vec(vec: Vec<LocalImage>) -> Self {
        Self {
            path: None,
            local_images: Some(vec),
        }
    }
}
