use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Size},
};
use ratatui_image::protocol::StatefulProtocol;
use std::fmt::Write;
use tracing::error;

use crate::{
    image_processor::ImageProcessor,
    local_image::LocalImage,
    ui::{
        grid::{Grid, GridState},
        screen,
    },
};

pub struct LocalImages {
    grid_state: GridState,
    cell_size: Size,
    image_processor: ImageProcessor,
    slice_size: usize,
}

pub enum LocalImagesEvent {
    Continue,
    Exit(Option<LocalImage>),
}

impl LocalImages {
    pub fn new(img_processor: ImageProcessor, cell_size: Size) -> Self {
        Self {
            grid_state: GridState::new(),
            cell_size,
            image_processor: img_processor,
            slice_size: 0,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(frame.area());

        let buf_len = self
            .image_processor
            .get_shared_buffer()
            .with(|buf| buf.len());

        let protocol_count = self.image_processor.get_shared_buffer().with(|buf| {
            let mut count = 0;

            buf.pair_vec.iter().for_each(|(_, protocol)| {
                if protocol.is_some() {
                    count += 1;
                }
            });

            count
        });

        let range_start = (self.grid_state.cells_in_column() + self.grid_state.offset())
            .saturating_sub(1)
            * self.grid_state.cells_in_row();
        let range_end = range_start + self.grid_state.cells_in_row();

        let range = if protocol_count > range_end {
            (buf_len - self.grid_state.cells_in_row())..buf_len
        } else {
            range_start..range_end
        };

        if self.slice_size == 0 {
            self.slice_size = self.grid_state.cells_in_row() * self.grid_state.cells_in_column();
        } else if let Some(selected) = self.grid_state.selected()
            && self.grid_state.item_count() > 0
            && range.contains(&selected)
        {
            self.slice_size += self.grid_state.cells_in_row()
        }

        if self.slice_size > buf_len {
            self.slice_size = buf_len;
        }

        self.image_processor
            .get_shared_buffer()
            .with_slice(0..self.slice_size, |slice| {
                self.grid_state.update_item_count(slice.len());
                self.grid_state
                    .update_size(layout[1].as_size(), self.cell_size);

                for (local_image, protocol) in &mut *slice {
                    if protocol.is_none() {
                        self.image_processor.process(local_image.clone());
                    }
                }

                let mut protocol_vec: Vec<&mut StatefulProtocol> = slice
                    .iter_mut()
                    .filter_map(|(_, protocol)| protocol.as_mut())
                    .collect();

                let grid = Grid::new(&mut protocol_vec, self.cell_size);

                let mut bottom_string = String::from(" <hjkl/←↓↑→> - Move | <Enter> - Select ");

                if cfg!(debug_assertions) {
                    if let Some(selected) = self.grid_state.selected() {
                        let _ = write!(bottom_string, "| Index: <{}> ", selected);
                    } else {
                        bottom_string.push_str("| Index: <None> ");
                    };

                    let _ = write!(bottom_string, "| buf: <{}> <{}> ", buf_len, protocol_count);
                    let _ = write!(
                        bottom_string,
                        "| <{}> <{}> <{}> ",
                        range.start,
                        range.end,
                        range.contains(&self.grid_state.selected().unwrap_or(0))
                    );
                }

                frame.render_widget(screen::create_top_bar(), layout[0]);
                frame.render_stateful_widget(grid, layout[1], &mut self.grid_state);
                frame.render_widget(screen::create_bottom_bar(&bottom_string), layout[2]);
            });
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> Result<LocalImagesEvent> {
        tokio::select! {
            Some(event) = event_stream.next() => {
                match event {
                    Ok(event) => {
                        match event {
                            Event::Key(key) => Ok(self.match_key(key.code)),
                            Event::Resize(_, _) => {
                                self.grid_state.select(0);
                                self.grid_state.set_offset(0);

                                Ok(LocalImagesEvent::Continue)
                            }
                            _ => Ok(LocalImagesEvent::Continue),
                        }
                    }
                    Err(e) => {
                        error!("{e:#}");
                        Err(e.into())
                    },
                }
            }

            _ = self.image_processor.wait_for_update() => {
                Ok(LocalImagesEvent::Continue)
            }
        }
    }

    fn match_key(&mut self, key: KeyCode) -> LocalImagesEvent {
        match key {
            KeyCode::Char('q') => return LocalImagesEvent::Exit(None),
            KeyCode::Char('h') | KeyCode::Left => self.grid_state.move_left(),
            KeyCode::Char('j') | KeyCode::Down => self.grid_state.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.grid_state.move_up(),
            KeyCode::Char('l') | KeyCode::Right => self.grid_state.move_right(),
            KeyCode::Enter => {
                if let Some(index) = self.grid_state.selected() {
                    return LocalImagesEvent::Exit(Some(
                        self.image_processor
                            .get_shared_buffer()
                            .with(|buffer| buffer.pair_vec[index].0.clone()),
                    ));
                }
                return LocalImagesEvent::Exit(None);
            }
            _ => {}
        }

        LocalImagesEvent::Continue
    }
}
