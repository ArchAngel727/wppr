use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use crossterm::event::{Event, EventStream};
use futures::StreamExt;
use image::{DynamicImage, ImageReader};
use ratatui::{
    Terminal,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::Size,
    prelude::CrosstermBackend,
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use std::{
    io::{Stdout, stdout},
    path::{Path, PathBuf},
};
use tokio::{fs, select, sync::mpsc};

use crate::{
    grid::{Grid, GridState},
    image_buffer::ImageBuffer,
    local_image::LocalImage,
};

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    picker: Option<Picker>,
}

impl Ui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let picker = Picker::from_query_stdio()?;

        Ok(Self {
            terminal,
            picker: Some(picker),
        })
    }

    pub fn load_images_from_fs(path: PathBuf) -> mpsc::Receiver<LocalImage> {
        let (tx, rx) = mpsc::channel::<LocalImage>(16);

        tokio::spawn(async move {
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
    ) -> mpsc::Receiver<(Result<DynamicImage>, DateTime<Utc>)> {
        let (tx, rx) = mpsc::channel::<(Result<DynamicImage>, DateTime<Utc>)>(16);

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

                    let _ = tx.blocking_send((result, image.date));
                });
            }
        });

        rx
    }

    fn create_protocol_from_image(
        mut image_tx: mpsc::Receiver<(Result<DynamicImage>, DateTime<Utc>)>,
        picker: Picker,
    ) -> mpsc::Receiver<(Result<StatefulProtocol>, DateTime<Utc>)> {
        let (tx, rx) = mpsc::channel::<(Result<StatefulProtocol>, DateTime<Utc>)>(16);

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

    pub async fn draw_grid(&mut self, path: &Path) -> Option<usize> {
        // TODO: Load cell_size from config file
        // let cell_size = Size::new(31, 10);
        // rewrite the thread chain to build up a ImageBufferItem instead of using pairs
        // add storing the path of an image to the ImageBuffer
        // return the path of the selected image instead of the selected index
        let cell_size = Size::new(20, 7);
        let mut grid_state = GridState::new();
        let mut events = EventStream::new();
        let mut images = ImageBuffer::new();

        let local_image_rx = Ui::load_images_from_fs(path.to_path_buf());
        let dynamic_image_rx = Ui::load_images_from_file(local_image_rx);
        let mut protocol_rx = Ui::create_protocol_from_image(
            dynamic_image_rx,
            self.picker.take().expect("Picker already taken"),
        );

        loop {
            let _ = self.terminal.draw(|frame| {
                let protocol_count = images.len();
                let grid = Grid::new(&mut images.protocols, cell_size);

                grid_state.update_item_count(protocol_count);
                grid_state.update_size(&frame.area().as_size(), &cell_size);

                frame.render_stateful_widget(grid, frame.area(), &mut grid_state);
            });

            select! {
                Some(maybe_event) = events.next() => {
                    if let Ok(Event::Key(key)) = maybe_event {
                        match key.code {
                            KeyCode::Char('q') => return None,
                            KeyCode::Char('h') => grid_state.move_left(),
                            KeyCode::Char('j') => grid_state.move_down(),
                            KeyCode::Char('k') => grid_state.move_up(),
                            KeyCode::Char('l') => grid_state.move_right(),
                            KeyCode::Enter => return grid_state.selected(),
                            _ => {},
                        }
                    }
                }

                Some((protocol, date_time)) = protocol_rx.recv() => {
                    if let Ok(protocol) = protocol {
                        images.protocols.push(protocol);
                        images.timestamps.push(date_time);
                        images.sort_by_timestamp();
                    }
                }
            }
        }
    }
}

impl Drop for Ui {
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
