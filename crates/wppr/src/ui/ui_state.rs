use crate::ui::{
    Screen,
    screen::{local_images::LocalImages, scrape_images::ScrapeImages, start::Start},
};

pub struct UiState {
    pub screen: Screen,
    pub start: Start,
    pub local_images: Option<LocalImages>,
    pub scrape_images: Option<ScrapeImages>,
}

impl UiState {
    pub const fn new() -> Self {
        Self {
            screen: Screen::Start,
            start: Start::new(),
            local_images: None,
            scrape_images: None,
        }
    }
}
