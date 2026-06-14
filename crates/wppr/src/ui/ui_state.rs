use crate::ui::{
    Screen,
    screen::{
        local_images::LocalImages, scrape_images::ScrapeImages, start::Start, too_small::TooSmall,
    },
};

pub struct UiState {
    pub screen: Screen,
    pub prev_screen: Option<Screen>,
    pub start: Start,
    pub too_small: TooSmall,
    pub local_images: Option<LocalImages>,
    pub scrape_images: Option<ScrapeImages>,
}

impl UiState {
    pub const fn new() -> Self {
        Self {
            screen: Screen::Start,
            prev_screen: None,
            start: Start::new(),
            too_small: TooSmall::new(),
            local_images: None,
            scrape_images: None,
        }
    }
}
