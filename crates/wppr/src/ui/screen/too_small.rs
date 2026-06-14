use ratatui::{Frame, widgets::Paragraph};

pub struct TooSmall {}

impl TooSmall {
    pub fn render(frame: &mut Frame) {
        frame.render_widget(Paragraph::new("Too small"), frame.area());
    }
}
