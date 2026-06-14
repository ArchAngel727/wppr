use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{Frame, widgets::Paragraph};
use tracing::error;

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
        match event_stream.next().await {
            Some(Ok(event)) => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => Ok(TooSmallEvent::Exit),
                    _ => Ok(TooSmallEvent::Continue),
                },
                Event::Resize(_, _) => Ok(TooSmallEvent::Continue),
                _ => Ok(TooSmallEvent::Continue),
            },
            None => Ok(TooSmallEvent::Continue),
            Some(Err(e)) => {
                error!("{e:#}");
                Err(e.into())
            }
        }
    }
}
