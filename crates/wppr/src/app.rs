use anyhow::{Result, anyhow};
use awww::{AwwwController, AwwwSocketStatus};
use hyprpanel::HyprpanelController;
use matugen::MatugenController;
use std::path::Path;
use tracing::{error, info};

use crate::cli::Cli;
use crate::config_manager::ConfigManager;
use crate::image_processor::ImageProcessorArgs;
use crate::scraper::Scraper;
use crate::ui::{Ui, event::UiResult, screen::Screen};
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
                AwwwController::set_wallpaper(&self.config.current_wallpaper)
                    .inspect_err(|e| error!("{e:#}"))?;
                MatugenController::update_colors(&self.config.current_wallpaper)
                    .inspect_err(|e| error!("{e:#}"))?;
            }
            AwwwSocketStatus::NotRunning => {
                if let Some(path) = self.config.current_wallpaper.to_str() {
                    let _ = wallpaper::set_from_path(path);
                }
            }
        }

        match HyprpanelController::check_daemon_status() {
            hyprpanel::HyprpanelSocketStatus::Running => {
                HyprpanelController::set_wallpaper(&self.config.current_wallpaper)
                    .inspect_err(|e| error!("{e:#}"))?;
            }
            hyprpanel::HyprpanelSocketStatus::NotRunning => (),
        }

        Ok(())
    }

    pub fn reload_wallpaper(&self) -> Result<()> {
        if !self.config.current_wallpaper.exists() {
            let e = anyhow!("No wallpaper selected");
            error!("{e}");
            return Err(anyhow!(e));
        }

        self.set_wallpaper().inspect_err(|e| error!("{e:#}"))?;

        Ok(())
    }

    fn match_ui_result(&mut self, selected: Result<UiResult>) -> Result<()> {
        match selected.inspect_err(|e| error!("{e:#}"))? {
            UiResult::Selected(local_image) => {
                self.config.current_wallpaper = local_image.path.clone();
                info!(
                    "Setting wallpaper {}",
                    self.config.current_wallpaper.display()
                );
                self.set_wallpaper().inspect_err(|e| error!("{e:#}"))?;
            }
            UiResult::Cancelled => {}
        }

        Ok(())
    }

    pub async fn menu(&mut self) -> Result<()> {
        let mut ui = Ui::new(&self.config)?;

        match &self.args.command {
            None => {
                let result = ui.run(None, None).await;

                drop(ui);

                self.match_ui_result(result)?;
            }
            Some(cli::Commands::Reload) => {
                self.reload_wallpaper().inspect_err(|e| error!("{e:#}"))?
            }
            Some(cli::Commands::Pick) => {
                let args = ImageProcessorArgs::from_path(self.config.save_dir.clone());
                let result = ui.run(Some(Screen::LocalImages), Some(args)).await;

                drop(ui);

                self.match_ui_result(result)?;
            }
            Some(cli::Commands::Scrape {
                tag,
                backstep,
                pick,
            }) => {
                let scraped_local_images =
                    Scraper::scrape_loacl_images(&self.config.save_dir, tag, backstep)
                        .await
                        .inspect_err(|e| error!("{e:#}"))?;

                if *pick {
                    let args = ImageProcessorArgs::from_local_images(scraped_local_images);
                    let result = ui.run(Some(Screen::ScrapeImages), Some(args)).await;
                    drop(ui);

                    self.match_ui_result(result)?;
                    return Ok(());
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
