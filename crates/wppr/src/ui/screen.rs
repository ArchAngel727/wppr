pub mod local_images;
pub mod options;
pub mod scrape_images;
pub mod start;
pub mod too_small;

use ratatui::{
    layout::{Constraint, Flex, Layout, Rect, Size},
    text::Line,
    widgets::{Block, Borders},
};

pub const MIN_SIZE: Size = Size::new(80, 23);

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Screen {
    Start,
    TooSmall,
    LocalImages,
    ScrapeImages,
    Options,
}

pub fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical(vec![Constraint::Length(height)])
        .flex(Flex::Center)
        .split(area);

    Layout::horizontal(vec![Constraint::Length(width)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}

pub fn create_top_bar() -> Block<'static> {
    Block::new()
        .title(Line::from(" Wppr ").centered())
        .borders(Borders::TOP)
}

pub fn create_bottom_bar(str: &str) -> Block<'_> {
    Block::new()
        .title(Line::from(str).centered())
        .borders(Borders::TOP)
}
