use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout},
    style::{Color, Stylize},
    widgets::{Block, Paragraph},
};

use crate::ui::screen;

pub struct Options {
    selected: usize,
}

pub enum OptionsEvent {
    Continue,
    Back,
    Exit(Option<usize>),
}

impl Options {
    pub fn new(selected: usize) -> Self {
        Self { selected }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let outer_layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(frame.area());

        let inner_layout = Layout::horizontal(vec![
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(outer_layout[1]);

        let inner_layout = [
            screen::center_rect(inner_layout[0], 12, 5),
            screen::center_rect(inner_layout[1], 20, 7),
            screen::center_rect(inner_layout[2], 31, 10),
        ];

        let mut cells = [
            Paragraph::new("Small Cell")
                .block(Block::bordered().fg(Color::Black))
                .centered(),
            Paragraph::new("Medium Cell")
                .block(Block::bordered().fg(Color::Black))
                .centered(),
            Paragraph::new("Large Cell")
                .block(Block::bordered().fg(Color::Black))
                .centered(),
        ];

        cells[self.selected] = cells[self.selected]
            .clone()
            .block(Block::bordered().fg(Color::White));

        frame.render_widget(screen::create_top_bar(), outer_layout[0]);
        frame.render_widget(
            screen::create_bottom_bar(" <h l / ← →> - Move | <Tab> - Cycle | <Enter> - Select "),
            outer_layout[2],
        );

        frame.render_widget(&cells[0], inner_layout[0]);
        frame.render_widget(&cells[1], inner_layout[1]);
        frame.render_widget(&cells[2], inner_layout[2]);
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> OptionsEvent {
        let Some(Ok(Event::Key(key))) = event_stream.next().await else {
            return OptionsEvent::Continue;
        };

        match key.code {
            KeyCode::Char('q') => return OptionsEvent::Exit(None),
            KeyCode::Char('h') | KeyCode::Left => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                if self.selected < 2 {
                    self.selected += 1;
                }
            }
            KeyCode::Tab => self.selected = (self.selected + 1) % 3,
            KeyCode::BackTab => {
                if self.selected == 0 {
                    self.selected = 2
                } else {
                    self.selected -= 1
                }
            }
            KeyCode::Enter => return OptionsEvent::Exit(Some(self.selected)),
            KeyCode::Backspace => return OptionsEvent::Back,
            _ => {}
        }

        OptionsEvent::Continue
    }
}
