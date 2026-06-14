pub mod local_images;
pub mod scrape_images;
pub mod start;
pub mod too_small;

use ratatui::layout::{Constraint, Flex, Layout, Rect, Size};

pub const MIN_SIZE: Size = Size::new(80, 23);

pub enum Screen {
    Start,
    LocalImages,
    ScrapeImages,
    TooSmall,
}

pub fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical(vec![Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);

    Layout::horizontal(vec![Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
