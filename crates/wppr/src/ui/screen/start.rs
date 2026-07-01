use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::Color,
    widgets::{Block, Paragraph},
};

use crate::ui::screen;

pub struct Start {
    selected: usize,
}

pub enum StartEvent {
    Continue,
    Exit(Option<usize>),
}

impl Start {
    pub const fn new() -> Self {
        Self { selected: 0 }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let outer_layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(frame.area());

        let inner_layout =
            Layout::horizontal(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(outer_layout[1]);

        let (left_color, right_color) = if self.selected == 0 {
            (Color::White, Color::Black)
        } else {
            (Color::Black, Color::White)
        };

        let left_block = Paragraph::new("Local Images")
            .block(Block::bordered().border_style(left_color))
            .centered();
        let right_block = Paragraph::new("Scrape Images")
            .block(Block::bordered().border_style(right_color))
            .centered();

        let left = screen::center_rect(inner_layout[0], 18, 7);
        let right = screen::center_rect(inner_layout[1], 18, 7);

        frame.render_widget(screen::create_top_bar(), outer_layout[0]);
        frame.render_widget(
            screen::create_bottom_bar(
                " <h l / ← →> - Move | <Tab> - Cycle | <Enter> - Select | <o> - Options ",
            ),
            outer_layout[2],
        );

        frame.render_widget(left_block, left);
        frame.render_widget(right_block, right);
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> StartEvent {
        let Some(Ok(Event::Key(key))) = event_stream.next().await else {
            return StartEvent::Continue;
        };

        match key.code {
            KeyCode::Char('q') => return StartEvent::Exit(None),
            KeyCode::Char('h') | KeyCode::Left => self.selected = 0,
            KeyCode::Char('l') | KeyCode::Right => self.selected = 1,
            KeyCode::Char('o') => return StartEvent::Exit(Some(2_usize)),
            KeyCode::Tab => self.selected = (self.selected + 1) % 2,
            KeyCode::Enter => return StartEvent::Exit(Some(self.selected)),
            _ => {}
        }

        StartEvent::Continue
    }
}
