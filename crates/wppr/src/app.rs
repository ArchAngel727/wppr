use anyhow::{Result, anyhow};
use awww::AwwwController;
use image::DynamicImage;
use matugen::MatugenController;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::Path;

use crate::cli::Cli;
use crate::config_manager::ConfigManager;
use crate::picker::Picker;
use crate::scraper::Scraper;
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
        AwwwController::set_wallpaper(&self.config.current_wallpaper)?;
        MatugenController::update_colors(&self.config.current_wallpaper)?;

        Ok(())
    }

    pub fn reload_wallpaper(&self) -> Result<()> {
        if !self.config.current_wallpaper.exists() {
            println!("{}", self.config.current_wallpaper.display());
            return Err(anyhow!("No wallpaper selected"));
        }

        println!("{}", self.config.current_wallpaper.display());
        self.set_wallpaper()?;

        Ok(())
    }

    pub async fn menu(&mut self) -> Result<()> {
        let mut url = "https://wallpaper-a-day.com".to_string();

        match &self.args.command {
            cli::Commands::Reload => self.reload_wallpaper()?,
            cli::Commands::Scrape {
                tag,
                backstep,
                pick,
            } => {
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

                self.config.current_wallpaper = if pick {
                    let images_clone = local_images.clone();

                    let images = tokio::task::spawn_blocking(move || {
                        images_clone
                            .par_iter()
                            .map(image::open)
                            .collect::<Result<Vec<DynamicImage>, _>>()
                    })
                    .await??;

                    let picker = Picker::new(&local_images, &images);

                    local_images[picker?.run()?].path.clone()
                } else {
                    local_images[0].path.clone()
                };

                self.set_wallpaper()?;
                ConfigManager::save_config(self)?;
            }
        };

        Ok(())
    }
}
