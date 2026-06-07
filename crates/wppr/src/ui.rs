use anyhow::Result;
use async_stream::try_stream;
use chrono::{DateTime, Utc};
use crossterm::event::{Event, EventStream};
use futures::{Stream, StreamExt, pin_mut};
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
use std::{
    io::{Stdout, stdout},
    path::Path,
};
use tokio::{fs, select};

use crate::{
    grid::{Grid, GridState},
    local_image::LocalImage,
};

pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Ui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        // let terminal_size = terminal.size()?;
        // let [_, right] =
        //     Self::split_layout(Rect::new(0, 0, terminal_size.width, terminal_size.height));

        Ok(Self { terminal })
    }

    pub fn load_images(path: &Path) -> impl Stream<Item = Result<LocalImage>> {
        try_stream! {
            let mut entries = fs::read_dir(path).await?;

            while let Some(item) = entries.next_entry().await? {
                let created_at: DateTime<Utc> = item.metadata().await?.created()?.into();
                yield LocalImage::from((item.path(), created_at));
            }
        }
    }

    pub async fn draw_grid(&mut self, path: &Path) -> Result<()> {
        let stream = Ui::load_images(path);
        let mut imgs: Vec<LocalImage> = Vec::new();
        let mut grid_state = GridState::new();
        let mut events = EventStream::new();

        pin_mut!(stream);

        loop {
            let _ = self.terminal.draw(|frame| {
                let grid_size = Size::new(20, 6);
                let grid = Grid::new(&imgs, grid_size);

                grid_state.update_item_count(imgs.len());
                grid_state.update_column_count((frame.area().width / grid_size.width) as usize);

                frame.render_stateful_widget(grid, frame.area(), &mut grid_state);
            });

            select! {
                maybe_event = events.next() => {
                    match maybe_event {
                        Some(Ok(Event::Key(key))) => {
                            match key.code {
                                KeyCode::Char('q') => break,
                                KeyCode::Char('h') => grid_state.move_left(),
                                KeyCode::Char('j') => grid_state.move_down(),
                                KeyCode::Char('k') => grid_state.move_up(),
                                KeyCode::Char('l') => grid_state.move_right(),
                                KeyCode::Char('p') => println!("{:?}", grid_state.selected()),
                                _ => {},
                            }
                        },
                        Some(Ok(_)) => {},
                        Some(Err(_)) => {},
                        None => {},
                    }
                }

                maybe_img = stream.next() => {
                    match maybe_img {
                        Some(Ok(img)) => imgs.push(img),
                        Some(Err(_)) => {},
                        None => {}
                    }
                }
            }
        }

        Ok(())
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
