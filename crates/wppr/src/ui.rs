pub mod event;
mod grid;
pub mod screen;
mod ui_state;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Frame, Terminal, prelude::CrosstermBackend};
use ratatui_image::picker::Picker;
use std::io::{Stdout, stdout};
use tracing::{error, info};

use crate::{
    config::{self, Config},
    image_processor::ImageProcessorArgs,
    scraper::Scraper,
    ui::{
        event::{EventResult, UiResult},
        screen::{
            MIN_SIZE, Screen, local_images::LocalImages, options::Options,
            scrape_images::ScrapeImages,
        },
        ui_state::UiState,
    },
};

pub struct Ui<'a> {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    picker: Option<Picker>,
    state: UiState,
    event_stream: EventStream,
    config: &'a mut Config,
}

impl<'a> Ui<'a> {
    pub fn new(config: &'a mut Config) -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let picker = Picker::from_query_stdio()?;

        Ok(Self {
            terminal,
            picker: Some(picker),
            state: UiState::new(),
            event_stream: EventStream::new(),
            config,
        })
    }

    pub async fn run(
        &mut self,
        screen: Option<Screen>,
        img_args: Option<ImageProcessorArgs>,
    ) -> Result<UiResult> {
        self.terminal.clear()?;

        if let Some(screen) = screen {
            self.state.screen = screen;
        }

        loop {
            self.terminal
                .draw(|frame| Self::draw_screen(frame, &mut self.state))?;

            match self.handle_events(img_args.clone()).await {
                Ok(event_result) => match event_result {
                    EventResult::Continue => {}
                    EventResult::Exit(ui_result) => {
                        let Some(ui_result) = ui_result else { continue };

                        return Ok(ui_result);
                    }
                    EventResult::Cancel => return Ok(UiResult::Cancelled),
                },
                Err(e) => {
                    error!("{e:#}");
                    return Err(e);
                }
            }
        }
    }

    fn draw_screen(frame: &mut Frame, state: &mut UiState) {
        let frame_size = frame.area();

        if frame_size.width < MIN_SIZE.width || frame_size.height < MIN_SIZE.height {
            state.prev_screen = Some(state.screen);
            state.screen = Screen::TooSmall;
        } else if let Some(prev_screen) = state.prev_screen {
            state.screen = prev_screen;
            state.prev_screen = None;
        }

        match state.screen {
            Screen::Start => state.start.draw(frame),
            Screen::LocalImages => {
                if let Some(screen) = &mut state.local_images {
                    screen.draw(frame);
                }
            }
            Screen::ScrapeImages => {
                if let Some(screen) = &mut state.scrape_images {
                    screen.draw(frame);
                }
            }
            Screen::TooSmall => state.too_small.draw(frame),
            Screen::Options => {
                if let Some(screen) = &mut state.options {
                    screen.draw(frame);
                }
            }
        }
    }

    async fn handle_events(&mut self, img_args: Option<ImageProcessorArgs>) -> Result<EventResult> {
        match self.state.screen {
            Screen::Start => match self.state.start.event(&mut self.event_stream).await {
                screen::start::StartEvent::Continue => Ok(EventResult::Continue),
                screen::start::StartEvent::Exit(Some(selected)) => {
                    #[cfg(debug_assertions)]
                    info!("Selected: {selected}");

                    match selected {
                        0 => {
                            let img_args = if let Some(args) = img_args {
                                args
                            } else {
                                ImageProcessorArgs::from_path(self.config.save_dir.clone())
                            };

                            self.state.screen = Screen::LocalImages;
                            self.state.local_images = Some(LocalImages::new(
                                self.picker.take().expect("Picker already taken"),
                                img_args,
                                config::CELL_SIZE[self.config.cell_size],
                            ));
                        }
                        1 => {
                            let img_args = if let Some(args) = img_args {
                                args
                            } else {
                                let local_images =
                                    Scraper::scrape_loacl_images(&self.config.save_dir, None, None)
                                        .await?;

                                ImageProcessorArgs::from_local_images(local_images)
                            };

                            self.state.screen = Screen::ScrapeImages;
                            self.state.scrape_images = Some(ScrapeImages::new(
                                self.picker.take().expect("Picker already taken"),
                                img_args,
                                config::CELL_SIZE[self.config.cell_size],
                            ));
                        }
                        2 => {
                            self.state.screen = Screen::Options;
                            self.state.options = Some(Options::new(self.config.cell_size));
                        }
                        _ => unreachable!("selected can only be between 0 and 2 inclusive"),
                    }

                    Ok(EventResult::Exit(None))
                }
                screen::start::StartEvent::Exit(None) => Ok(EventResult::Cancel),
            },

            Screen::LocalImages => {
                let Some(local_images_screen) = &mut self.state.local_images else {
                    return Ok(EventResult::Continue);
                };

                match local_images_screen.event(&mut self.event_stream).await {
                    screen::local_images::LocalImagesEvent::Continue => Ok(EventResult::Continue),
                    screen::local_images::LocalImagesEvent::Exit(Some(local_image)) => {
                        Ok(EventResult::Exit(Some(UiResult::Selected(local_image))))
                    }
                    screen::local_images::LocalImagesEvent::Exit(None) => {
                        Ok(EventResult::Exit(Some(UiResult::Cancelled)))
                    }
                }
            }

            Screen::ScrapeImages => {
                let Some(scrape_images_screen) = &mut self.state.scrape_images else {
                    return Ok(EventResult::Continue);
                };

                match scrape_images_screen.event(&mut self.event_stream).await {
                    screen::scrape_images::ScrapeImagesEvent::Continue => Ok(EventResult::Continue),
                    screen::scrape_images::ScrapeImagesEvent::Exit(Some(local_image)) => {
                        Ok(EventResult::Exit(Some(UiResult::Selected(local_image))))
                    }
                    screen::scrape_images::ScrapeImagesEvent::Exit(None) => {
                        Ok(EventResult::Exit(Some(UiResult::Cancelled)))
                    }
                }
            }

            Screen::TooSmall => match self.state.too_small.event(&mut self.event_stream).await {
                screen::too_small::TooSmallEvent::Continue => Ok(EventResult::Continue),
                screen::too_small::TooSmallEvent::Exit => {
                    Ok(EventResult::Exit(Some(UiResult::Cancelled)))
                }
            },
            Screen::Options => {
                let Some(options_screen) = &mut self.state.options else {
                    return Ok(EventResult::Continue);
                };

                match options_screen.event(&mut self.event_stream).await {
                    screen::options::OptionsEvent::Continue => Ok(EventResult::Continue),
                    screen::options::OptionsEvent::Back => {
                        self.go_back(Screen::Start);

                        Ok(EventResult::Continue)
                    }
                    screen::options::OptionsEvent::Exit(Some(selected)) => {
                        #[cfg(debug_assertions)]
                        info!("Selected cell size: {}", selected);

                        self.config.cell_size = selected;
                        self.go_back(Screen::Start);

                        Ok(EventResult::Continue)
                    }
                    screen::options::OptionsEvent::Exit(None) => {
                        Ok(EventResult::Exit(Some(UiResult::Cancelled)))
                    }
                }
            }
        }
    }

    fn go_back(&mut self, screen: Screen) {
        self.state.screen = if let Some(prev_screen) = self.state.prev_screen {
            prev_screen
        } else {
            screen
        };

        self.state.prev_screen = None;
    }
}

impl Drop for Ui<'_> {
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
