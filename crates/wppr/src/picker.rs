use anyhow::Result;
use image::DynamicImage;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{List, ListItem, ListState},
};
use ratatui_image::{Image, Resize, picker::Picker as RatatuiPicker, protocol::Protocol};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    fmt::Display,
    io::{Stdout, stdout},
    path::Path,
};

pub struct PickerState {
    value: usize,
    max: usize,
    list_state: ListState,
}

impl PickerState {
    pub fn new(len: usize) -> Self {
        Self {
            value: 0,
            max: len,
            list_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn next(&mut self) {
        if self.value == self.max {
            self.value = 0;
        } else {
            self.value += 1;
        }

        self.list_state.select(Some(self.value));
    }

    pub fn previous(&mut self) {
        if self.value == 0 {
            self.value = self.max;
        } else {
            self.value -= 1;
        }

        self.list_state.select(Some(self.value));
    }
}

pub struct Picker<'a, T: Display + AsRef<Path>> {
    state: PickerState,
    values: &'a [T],
    protocols: Vec<Protocol>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl<'a, T: Display + AsRef<Path>> Picker<'a, T> {
    pub fn new(values: &'a [T], images: &'a [DynamicImage]) -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let picker = RatatuiPicker::from_query_stdio()?;
        let terminal_size = terminal.size()?;
        let [_, right] =
            Self::split_layout(Rect::new(0, 0, terminal_size.width, terminal_size.height));

        let protocols: Vec<Protocol> = images
            .par_iter()
            .flat_map(|img| picker.new_protocol(img.clone(), right, Resize::Fit(None)))
            .collect();

        Ok(Self {
            state: PickerState::new(values.len() - 1),
            values,
            protocols,
            terminal,
        })
    }

    fn split_layout(area: Rect) -> [Rect; 2] {
        let constraints = [Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)];
        let layout = Layout::horizontal(constraints).spacing(1);
        area.layout(&layout)
    }

    pub fn run(&mut self) -> Result<Option<usize>> {
        for p in &mut self.protocols {
            self.terminal.draw(|frame| {
                frame.render_widget(Image::new(p), Rect::new(0, 0, 1, 1));
            })?;
        }
        self.terminal.clear()?;

        loop {
            self.terminal.draw(|frame| {
                let [left, right] = Self::split_layout(frame.area());
                let list_items: Vec<ListItem> = self
                    .values
                    .iter()
                    .map(|item| ListItem::new(item.to_string()))
                    .collect();

                let list = List::new(list_items)
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::new().blue().bold())
                    .highlight_symbol("> ");

                let widget = Image::new(&self.protocols[self.state.value]);

                frame.render_stateful_widget(list, left, &mut self.state.list_state);
                frame.render_widget(widget, right);
            })?;

            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('q') => return Ok(None),
                    KeyCode::Char('j') => self.state.next(),
                    KeyCode::Char('k') => self.state.previous(),
                    KeyCode::Enter => return Ok(Some(self.state.value)),
                    _ => {}
                }
            }
        }
    }
}

impl<'a, T: Display + AsRef<Path>> Drop for Picker<'a, T> {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
