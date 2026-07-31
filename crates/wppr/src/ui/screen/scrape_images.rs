use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Size},
};
use ratatui_image::protocol::StatefulProtocol;
use tracing::error;

use crate::{
    image_processor::ImageProcessor,
    local_image::LocalImage,
    ui::{
        grid::{Grid, GridState},
        screen,
    },
};

pub struct ScrapeImages {
    grid_state: GridState,
    cell_size: Size,
    image_processor: ImageProcessor,
}

pub enum ScrapeImagesEvent {
    Continue,
    Exit(Option<LocalImage>),
}

impl ScrapeImages {
    pub fn new(img_processor: ImageProcessor, cell_size: Size) -> Self {
        Self {
            grid_state: GridState::new(),
            cell_size,
            image_processor: img_processor,
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

        let slice_size = self.grid_state.cells_in_row();

        self.image_processor
            .get_shared_buffer()
            .with_slice(0..slice_size, |slice| {
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

                frame.render_widget(screen::create_top_bar(), layout[0]);
                frame.render_stateful_widget(grid, layout[1], &mut self.grid_state);
                frame.render_widget(
                    screen::create_bottom_bar(" <hjkl/←↓↑→> - Move | <Enter> - Select "),
                    layout[2],
                );
            });
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> Result<ScrapeImagesEvent> {
        tokio::select! {
            Some(event) = event_stream.next() => {
                match event {
                    Ok(event) => {
                        match event {
                            Event::Key(key) => Ok(self.match_key(key.code)),
                            Event::Resize(_, _) => {
                                self.grid_state.select(0);
                                self.grid_state.set_offset(0);

                                Ok(ScrapeImagesEvent::Continue)
                            }
                            _ => Ok(ScrapeImagesEvent::Continue),
                        }
                    }
                    Err(e) => {
                        error!("{e:#}");
                        Err(e.into())
                    },
                }
            }

            _ = self.image_processor.wait_for_update() => {
                Ok(ScrapeImagesEvent::Continue)
            }
        }
    }

    fn match_key(&mut self, key: KeyCode) -> ScrapeImagesEvent {
        match key {
            KeyCode::Char('q') => return ScrapeImagesEvent::Exit(None),
            KeyCode::Char('h') | KeyCode::Left => self.grid_state.move_left(),
            KeyCode::Char('j') | KeyCode::Down => self.grid_state.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.grid_state.move_up(),
            KeyCode::Char('l') | KeyCode::Right => self.grid_state.move_right(),
            KeyCode::Enter => {
                if let Some(index) = self.grid_state.selected() {
                    return ScrapeImagesEvent::Exit(Some(
                        self.image_processor
                            .get_shared_buffer()
                            .with(|buffer| buffer.pair_vec[index].0.clone()),
                    ));
                }
                return ScrapeImagesEvent::Exit(None);
            }
            _ => {}
        }

        ScrapeImagesEvent::Continue
    }
}
