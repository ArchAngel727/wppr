use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode};
use futures::StreamExt;
use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Size},
};
use ratatui_image::picker::Picker;
use tracing::error;

use crate::{
    image_buffer::ImageBuffer,
    image_processor::{ImageProcessor, ImageProcessorArgs},
    local_image::LocalImage,
    ui::{
        grid::{Grid, GridState},
        screen,
    },
};

pub struct ScrapeImages {
    grid_state: GridState,
    cell_size: Size,
    image_buffer: ImageBuffer,
    image_processor: ImageProcessor,
}

pub enum ScrapeImagesEvent {
    Continue,
    Exit(Option<LocalImage>),
}

impl ScrapeImages {
    pub fn new(picker: Picker, args: ImageProcessorArgs, cell_size: Size) -> Self {
        Self {
            grid_state: GridState::new(),
            cell_size,
            image_buffer: ImageBuffer::new(),
            image_processor: ImageProcessor::new(picker, args),
        }
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .split(frame.area());

        self.grid_state.update_item_count(self.image_buffer.len());
        self.grid_state
            .update_size(layout[1].as_size(), self.cell_size);

        let grid = Grid::new(&mut self.image_buffer.protocols, self.cell_size);

        frame.render_widget(screen::create_top_bar(), layout[0]);
        frame.render_stateful_widget(grid, layout[1], &mut self.grid_state);
        frame.render_widget(
            screen::create_bottom_bar(" <hjkl/←↓↑→> - Move | <Enter> - Select "),
            layout[2],
        );
    }

    pub async fn event(&mut self, event_stream: &mut EventStream) -> Result<ScrapeImagesEvent> {
        tokio::select! {
            Some(event) = event_stream.next() => {
                match event {
                    Ok(event) => {
                        match event {
                            Event::Key(key) => self.match_key(key.code),
                            Event::Resize(_, _) => {
                                self.grid_state.select(0);
                                self.grid_state.set_offset(0);

                                Ok(ScrapeImagesEvent::Continue)
                            }
                            _ => Ok(ScrapeImagesEvent::Continue),
                        }
                    }
                    Err(_) => todo!(),
                }
            }

            Some((protocol, local_image)) = self.image_processor.rx.recv() => {
                if let Ok(protocol) = protocol {
                    self.image_buffer.protocols.push(protocol);
                    self.image_buffer.local_images.push(local_image);
                    self.image_buffer.sort_by_timestamp();
                } else {
                    error!("Failed to load image: {}", local_image.path.display());
                }

                Ok(ScrapeImagesEvent::Continue)
            }
        }
    }

    fn match_key(&mut self, key: KeyCode) -> Result<ScrapeImagesEvent> {
        match key {
            KeyCode::Char('q') => return Ok(ScrapeImagesEvent::Exit(None)),
            KeyCode::Char('h') | KeyCode::Left => self.grid_state.move_left(),
            KeyCode::Char('j') | KeyCode::Down => self.grid_state.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.grid_state.move_up(),
            KeyCode::Char('l') | KeyCode::Right => self.grid_state.move_right(),
            KeyCode::Enter => {
                if let Some(index) = self.grid_state.selected() {
                    return Ok(ScrapeImagesEvent::Exit(Some(
                        self.image_buffer.local_images[index].clone(),
                    )));
                }
                return Ok(ScrapeImagesEvent::Exit(None));
            }
            _ => {}
        }

        Ok(ScrapeImagesEvent::Continue)
    }
}
