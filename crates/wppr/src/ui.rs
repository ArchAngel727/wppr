pub mod grid;
pub mod packet;

use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use crossterm::event::{self, Event, EventStream};
use futures::StreamExt;
use image::{DynamicImage, ImageReader};
use ratatui::{
    Terminal,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Flex, Layout, Rect, Size},
    prelude::CrosstermBackend,
    style::Color,
    text::Line,
    widgets::{Block, Borders, Paragraph},
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::io::{Stdout, stdout};
use tokio::{fs, select, sync::mpsc};
use tracing::error;

use crate::{
    config::Config,
    image_buffer::ImageBuffer,
    local_image::LocalImage,
    ui::grid::{Grid, GridState},
    ui::packet::Packet,
};

pub struct Ui<'a> {
    config: &'a Config,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    picker: Option<Picker>,
}

pub enum StartMenuSelection {
    LocalImages,
    ScrapeImages,
}

impl<'a> Ui<'a> {
    pub fn new(config: &'a Config) -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let picker = Picker::from_query_stdio()?;

        Ok(Self {
            config,
            terminal,
            picker: Some(picker),
        })
    }

    pub fn start_menu(&mut self) -> Result<Option<StartMenuSelection>> {
        let mut selected: usize = 0;

        // TODO: popup menu when <?> with help text

        loop {
            let _ = self.terminal.draw(|frame| {
                let outer_layout = Layout::vertical(vec![
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .flex(Flex::Center)
                .split(frame.area());

                let inner_layout = Layout::horizontal(vec![
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(outer_layout[1]);

                let (left_color, right_color) = if selected == 0 {
                    (Color::White, Color::Black)
                } else {
                    (Color::Black, Color::White)
                };

                let top_bar = Block::new()
                    .title(Line::from(" Wppr ").centered())
                    .borders(Borders::TOP);
                let bottom_bar = Block::new()
                    .title(
                        Line::from(" <h l / ← →> - Move | <Tab> - Cycle | <Enter> - Select ")
                            .centered(),
                    )
                    .borders(Borders::TOP);

                let left_block = Paragraph::new("Local Images")
                    .block(Block::bordered().border_style(left_color))
                    .centered();
                let right_block = Paragraph::new("Scrape Images")
                    .block(Block::bordered().border_style(right_color))
                    .centered();

                let left = Ui::center_rect(inner_layout[0], 18, 7);
                let right = Ui::center_rect(inner_layout[1], 18, 7);

                frame.render_widget(top_bar, outer_layout[0]);
                frame.render_widget(bottom_bar, outer_layout[2]);

                frame.render_widget(left_block, left);
                frame.render_widget(right_block, right);
            });

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break Ok(None),
                    KeyCode::Char('h') | KeyCode::Left => selected = 0,
                    KeyCode::Char('l') | KeyCode::Right => selected = 1,
                    KeyCode::Tab => selected = (selected + 1) % 2,
                    KeyCode::Enter => {
                        break Ok(Some(match selected {
                            0 => StartMenuSelection::LocalImages,
                            _ => StartMenuSelection::ScrapeImages,
                        }));
                    }
                    _ => {}
                }
            }
        }
    }

    fn center_rect(area: Rect, width: u16, height: u16) -> Rect {
        let vertical = Layout::vertical(vec![Constraint::Length(height)])
            .flex(Flex::Center)
            .split(area);

        Layout::horizontal(vec![Constraint::Length(width)])
            .flex(Flex::Center)
            .split(vertical[0])[0]
    }

    pub async fn picker_grid(&mut self, packet: Option<Packet>) -> Option<LocalImage> {
        // TODO: Load cell_size from config file
        // let cell_size = Size::new(31, 10);
        let cell_size = Size::new(20, 7);
        let mut grid_state = GridState::new();
        let mut events = EventStream::new();
        let mut images = ImageBuffer::new();

        let local_image_rx = Ui::load_images_from_fs(match packet {
            Some(packet) => packet,
            None => Packet::from_path(self.config.save_dir.clone()),
        });
        let dynamic_image_rx = Ui::load_images_from_file(local_image_rx);
        let mut protocol_rx = Ui::create_protocol_from_image(
            dynamic_image_rx,
            self.picker.take().expect("Picker already taken"),
        );

        loop {
            let _ = self.terminal.draw(|frame| {
                let layout = Layout::vertical(vec![
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .flex(Flex::Center)
                .split(frame.area());

                let top_bar = Block::new()
                    .title(Line::from(" Wppr ").centered())
                    .borders(Borders::TOP);
                let bottom_bar = Block::new()
                    .title(Line::from(" <hjkl/←↓↑→> - Move | <Enter> - Select ").centered())
                    .borders(Borders::TOP);

                let protocol_count = images.len();
                let grid = Grid::new(&mut images.protocols, cell_size);

                grid_state.update_item_count(protocol_count);
                grid_state.update_size(&layout[1].as_size(), &cell_size);

                frame.render_widget(top_bar, layout[0]);
                frame.render_stateful_widget(grid, layout[1], &mut grid_state);
                frame.render_widget(bottom_bar, layout[2]);
            });

            select! {
                Some(maybe_event) = events.next() => {
                    if let Ok(Event::Key(key)) = maybe_event {
                        match key.code {
                            KeyCode::Char('q') => return None,
                            KeyCode::Char('h') | KeyCode::Left => grid_state.move_left(),
                            KeyCode::Char('j') | KeyCode::Down => grid_state.move_down(),
                            KeyCode::Char('k') | KeyCode::Up => grid_state.move_up(),
                            KeyCode::Char('l') | KeyCode::Right => grid_state.move_right(),
                            KeyCode::Enter => {
                                    if let Some(index) = grid_state.selected() {
                                        return Some(images.local_images[index].clone());
                                    } else {
                                        return None;
                                    }
                                },
                            _ => {},
                        }
                    }
                }

                Some((protocol, local_image)) = protocol_rx.recv() => {
                    if let Ok(protocol) = protocol {
                        images.protocols.push(protocol);
                        images.local_images.push(local_image);
                        images.sort_by_timestamp();
                    } else {
                        error!("Failed to load image: {}", local_image.path.display());
                    }
                }
            }
        }
    }

    fn load_images_from_fs(packet: Packet) -> mpsc::Receiver<LocalImage> {
        let (tx, rx) = mpsc::channel::<LocalImage>(16);

        tokio::spawn(async move {
            if let Some(local_images) = packet.local_images {
                for local_image in local_images {
                    if tx.send(local_image).await.is_err() {
                        break;
                    }
                }
            }

            if packet.path.is_none() {
                return;
            }

            let path = packet.path.unwrap();

            let mut entries = match fs::read_dir(&path).await {
                Ok(entries) => entries,
                Err(e) => {
                    eprintln!("read_dir {}: {}", path.display(), e);
                    return;
                }
            };

            while let Ok(Some(entry)) = entries.next_entry().await {
                let metadata = match entry.metadata().await {
                    Ok(metadata) => metadata,
                    _ => continue,
                };

                let date: DateTime<Utc> = metadata
                    .modified()
                    .ok()
                    .map(DateTime::from)
                    .unwrap_or(Utc::now());

                let local_image = LocalImage::from((entry.path(), date));

                if tx.send(local_image).await.is_err() {
                    break;
                }
            }
        });

        rx
    }

    fn load_images_from_file(
        mut image_tx: mpsc::Receiver<LocalImage>,
    ) -> mpsc::Receiver<(Result<DynamicImage>, LocalImage)> {
        let (tx, rx) = mpsc::channel::<(Result<DynamicImage>, LocalImage)>(16);

        tokio::spawn(async move {
            while let Some(image) = image_tx.recv().await {
                let tx = tx.clone();

                tokio::task::spawn_blocking(move || {
                    let result = (|| -> Result<DynamicImage> {
                        ImageReader::open(&image.path)?
                            .with_guessed_format()?
                            .decode()
                            .map_err(|e| anyhow!("decode {}: {}", image.path.display(), e))
                    })();

                    let _ = tx.blocking_send((result, image));
                });
            }
        });

        rx
    }

    fn create_protocol_from_image(
        mut image_tx: mpsc::Receiver<(Result<DynamicImage>, LocalImage)>,
        picker: Picker,
    ) -> mpsc::Receiver<(Result<StatefulProtocol>, LocalImage)> {
        let (tx, rx) = mpsc::channel::<(Result<StatefulProtocol>, LocalImage)>(16);

        tokio::spawn(async move {
            while let Some(image) = image_tx.recv().await {
                let tx = tx.clone();
                let picker = picker.clone();

                tokio::task::spawn_blocking(move || {
                    let result = (|| -> Result<StatefulProtocol> {
                        let image = image.0?;

                        Ok(picker.new_resize_protocol(image))
                    })();

                    let _ = tx.blocking_send((result, image.1));
                });
            }
        });

        rx
    }
}

impl<'a> Drop for Ui<'a> {
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
