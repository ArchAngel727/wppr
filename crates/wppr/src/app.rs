use anyhow::{Result, anyhow};
use awww::{AwwwController, AwwwSocketStatus};
use hyprpanel::HyprpanelController;
use matugen::MatugenController;
use std::path::Path;
use tracing::{error, info};

use crate::cli::Cli;
use crate::config_manager::ConfigManager;
use crate::local_image::LocalImage;
use crate::scraper::Scraper;
use crate::ui::{Ui, packet::Packet};
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

    fn match_image(&mut self, image: &Option<LocalImage>) -> Result<()> {
        match image {
            Some(image) => {
                self.config.current_wallpaper = image.path.clone();
                self.set_wallpaper()?;
            }
            None => info!("No image was selected"),
        }

        Ok(())
    }

    async fn scrape_loacl_images(
        &self,
        tag: &Option<String>,
        url: &mut String,
        backstep: &Option<u32>,
    ) -> Result<Vec<LocalImage>> {
        let tags = Scraper::scrape_tags().await?;

        if let Some(tag) = tag {
            match tags.iter().find(|t| t.starts_with(tag)) {
                Some(tag) => {
                    url.push_str("/category/");
                    url.push_str(tag);
                }
                None => todo!(),
            }
        }

        Scraper::scrape(self, url, backstep.unwrap_or(0)).await
    }

    pub async fn menu(&mut self) -> Result<()> {
        let mut url = String::from("https://wallpaper-a-day.com");
        let mut ui = Ui::new(&self.config)?;

        match &self.args.command {
            None => {
                let selected = ui.start_menu();

                match selected {
                    Ok(Some(selected)) => {
                        if selected == 0 {
                            let image = ui.draw_grid(None).await;
                            drop(ui);

                            self.match_image(&image)?;
                        }
                    }
                    Ok(None) => info!("Nothing selected"),
                    Err(e) => error!("{}", e),
                }
            }
            Some(cli::Commands::Reload) => self.reload_wallpaper()?,
            Some(cli::Commands::Pick) => {
                let image = ui.draw_grid(None).await;
                drop(ui);

                self.match_image(&image)?;
            }
            Some(cli::Commands::Scrape {
                tag,
                backstep,
                pick,
            }) => {
                let scraped_local_images =
                    self.scrape_loacl_images(tag, &mut url, backstep).await?;

                if *pick {
                    let packet = Packet::from_img_vec(scraped_local_images);
                    let image = ui.draw_grid(Some(packet)).await;
                    drop(ui);

                    self.match_image(&image)?;
                } else {
                    drop(ui);
                    self.config.current_wallpaper = scraped_local_images[0].path.clone();
                }

                self.set_wallpaper()?;
            }
        };

        ConfigManager::save_config(self)?;
        Ok(())
    }
}
