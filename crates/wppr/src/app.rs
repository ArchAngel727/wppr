use anyhow::{Result, anyhow};
use awww::{AwwwController, AwwwSocketStatus};
use chrono::{DateTime, Utc};
use hyprpanel::HyprpanelController;
use image::DynamicImage;
use matugen::MatugenController;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{fs, path::Path};

use crate::cli::Cli;
use crate::config_manager::ConfigManager;
use crate::local_image::LocalImage;
use crate::picker::Picker;
use crate::scraper::Scraper;
use crate::ui::Ui;
use crate::{Config, cli};

pub struct App<'a> {
    pub config_path: &'a Path,
    pub config: Config,
    pub args: Cli,
}

impl<'a> App<'a> {
    pub fn new(config_path: &'a Path, config: Config, args: Cli) -> App<'a> {
        App {
            config_path,
            config,
            args,
        }
    }

    pub fn set_wallpaper(&self) -> Result<()> {
        match AwwwController::check_daemon_status() {
            AwwwSocketStatus::Running => {
                AwwwController::set_wallpaper(&self.config.current_wallpaper)?;
                MatugenController::update_colors(&self.config.current_wallpaper)?;
            }
            AwwwSocketStatus::NotRunning => {
                if let Some(path) = self.config.current_wallpaper.to_str() {
                    let _ = wallpaper::set_from_path(path);
                }
            }
        }

        match HyprpanelController::check_daemon_status() {
            hyprpanel::HyprpanelSocketStatus::Running => {
                HyprpanelController::set_wallpaper(&self.config.current_wallpaper)?;
            }
            hyprpanel::HyprpanelSocketStatus::NotRunning => (),
        }

        Ok(())
    }

    pub fn reload_wallpaper(&self) -> Result<()> {
        if !self.config.current_wallpaper.exists() {
            return Err(anyhow!("No wallpaper selected"));
        }

        self.set_wallpaper()?;

        Ok(())
    }

    fn list_dir(&self) -> Result<Vec<LocalImage>> {
        Ok(fs::read_dir(&self.config.save_dir)?
            .filter_map(|item| {
                let item = item.ok()?;
                let created_at: DateTime<Utc> = item.metadata().ok()?.created().ok()?.into();
                Some(LocalImage::from((item.path(), created_at)))
            })
            .collect::<Vec<LocalImage>>())
    }

    async fn load_images(images: Vec<LocalImage>) -> Result<Vec<DynamicImage>> {
        tokio::task::spawn_blocking(move || {
            images
                .par_iter()
                .map(|f| {
                    image::ImageReader::open(f)?
                        .with_guessed_format()?
                        .decode()
                        .map_err(|e| anyhow!("decode {}: {e}", f))
                })
                .collect::<Result<Vec<DynamicImage>, _>>()
        })
        .await?
    }

    pub async fn menu(&mut self) -> Result<()> {
        let mut url = "https://wallpaper-a-day.com".to_string();

        match &self.args.command {
            None => {
                let mut ui = Ui::new(&self.config)?;
                // let pick = ui.draw_grid().await;
                let selected = ui.start_menu();

                drop(ui);

                match selected {
                    Ok(Some(selected)) => println!("Selected: {}", selected),
                    Ok(None) => println!("Nothing selected"),
                    Err(e) => eprintln!("{}", e),
                }

                // if let Some(pick) = pick {
                //     self.config.current_wallpaper = pick.path.clone();
                //     self.set_wallpaper()?;
                // }
            }
            Some(cli::Commands::Reload) => self.reload_wallpaper()?,
            Some(cli::Commands::Pick) => {
                let mut local_images = self.list_dir()?;
                local_images.sort_by_key(|k| k.date);
                let images = App::load_images(local_images.clone()).await?;
                let mut picker = Picker::new(&local_images, &images)?;

                if let Ok(Some(result)) = picker.run() {
                    self.config.current_wallpaper = local_images[result].path.clone();

                    self.set_wallpaper()?;
                    ConfigManager::save_config(self)?;
                }
            }
            Some(cli::Commands::Scrape {
                tag,
                backstep,
                pick,
            }) => {
                let tags = Scraper::scrape_tags().await?;
                let pick = *pick;

                if let Some(tag) = tag {
                    match tags.iter().find(|t| t.starts_with(tag)) {
                        Some(tag) => {
                            url.push_str("/category/");
                            url.push_str(tag);
                        }
                        None => todo!(),
                    }
                }

                let local_images = Scraper::scrape(self, &url, backstep.unwrap_or(0)).await?;

                if pick {
                    let images = App::load_images(local_images.clone()).await?;
                    let mut picker = Picker::new(&local_images, &images)?;

                    if let Ok(Some(result)) = picker.run() {
                        self.config.current_wallpaper = local_images[result].path.clone();
                    }
                } else {
                    self.config.current_wallpaper = local_images[0].path.clone();
                }

                self.set_wallpaper()?;
                ConfigManager::save_config(self)?;
            }
        };

        Ok(())
    }
}
