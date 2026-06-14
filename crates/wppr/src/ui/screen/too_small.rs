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

    pub fn render(&self, frame: &mut Frame) {
        frame.render_widget(Paragraph::new("Too small"), frame.area());
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> TooSmallEvent {
        match event_stream.next().await {
            Some(Ok(event)) => match event {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => TooSmallEvent::Exit,
                    _ => TooSmallEvent::Continue,
                },
                Event::Resize(_, _) => TooSmallEvent::Continue,
                _ => TooSmallEvent::Continue,
            },
            None => TooSmallEvent::Continue,
            Some(Err(_)) => todo!(),
        }
    }
}
