use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{Frame, widgets::Paragraph};

pub struct TooSmall {}

pub enum TooSmallEvent {
    Continue,
    Exit,
}

impl TooSmall {
    pub const fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(Paragraph::new("Too small"), frame.area());
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> Result<TooSmallEvent> {
        let Some(Ok(Event::Key(key))) = event_stream.next().await else {
            return Ok(TooSmallEvent::Continue);
        };

        match key.code {
            KeyCode::Char('q') => Ok(TooSmallEvent::Exit),
            _ => Ok(TooSmallEvent::Continue),
        }
    }
}
