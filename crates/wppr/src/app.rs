use anyhow::{Result, anyhow};
use awww::{AwwwController, AwwwSocketStatus};
use matugen::MatugenController;
use std::path::Path;
use tracing::{error, info};
use wayle::WayleController;

use crate::{
    Config,
    cli::{self, Cli},
    ui::{Ui, event::UiResult, screen::Screen},
};

pub struct App<'a> {
    pub config_path: &'a Path,
    pub config: Config,
    pub args: Cli,
}

impl<'a> App<'a> {
    pub const fn new(config_path: &'a Path, config: Config, args: Cli) -> Self {
        App {
            config_path,
            config,
            args,
        }
    }

    pub fn set_wallpaper(&self) -> Result<()> {
        if AwwwController::check_daemon_status() == AwwwSocketStatus::Running {
            AwwwController::set_wallpaper(&self.config.current_wallpaper)
                .inspect_err(|e| error!("{e:#}"))?;

            self.update_colors()?;
            return Ok(());
        }

        if let Some(path) = self.config.current_wallpaper.to_str() {
            let _ = wallpaper::set_from_path(path).inspect_err(|e| error!("{e:#}"));
        }

        Ok(())
    }

    fn update_colors(&self) -> Result<()> {
        MatugenController::update_colors(&self.config.current_wallpaper)
            .inspect_err(|e| error!("{e:#}"))?;

        if WayleController::is_running() {
            WayleController::set_wallpaper(&self.config.current_wallpaper)?;
        }

        Ok(())
    }

    pub fn reload_wallpaper(&self) -> Result<()> {
        if !self.config.current_wallpaper.exists() {
            let e = anyhow!("No wallpaper selected");
            error!("{}", e);
            return Err(anyhow!(e));
        }

        self.set_wallpaper().inspect_err(|e| error!("{e:#}"))?;

        Ok(())
    }

    fn match_ui_result(&mut self, selected: Result<UiResult>) -> Result<()> {
        match selected.inspect_err(|e| error!("{e:#}"))? {
            UiResult::Selected(local_image) => {
                self.config.current_wallpaper = local_image.path;
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
        let save_dir = self.config.save_dir.clone();
        let mut config = self.config.clone();

        match &self.args.command {
            None => {
                let mut ui = Ui::new(&mut config)?;
                let result = ui.run(None, None).await;

                self.match_ui_result(result)?;
            }

            Some(cli::Commands::Reload) => {
                self.reload_wallpaper().inspect_err(|e| error!("{e:#}"))?;
            }

            Some(cli::Commands::Pick) => {
                let mut ui = Ui::new(&mut config)?;
                let result = ui.run(Some(Screen::LocalImages), Some(save_dir)).await;

                self.match_ui_result(result)?;
            }

            Some(cli::Commands::Scrape { tag: _, pick: _ }) => {} // TODO: reimplement tags
                                                                  // {
                                                                  //     let scraped_local_images =
                                                                  //         Scraper::scrape_images(&save_dir, tag.clone(), Some(1)).await?;
                                                                  //
                                                                  //     if *pick {
                                                                  //         let args = ImageProcessorArgs::from_local_images(scraped_local_images);
                                                                  //         let mut ui = Ui::new(&mut config)?;
                                                                  //         let result = ui.run(Some(Screen::ScrapeImages), Some(args)).await;
                                                                  //
                                                                  //         self.match_ui_result(result)?;
                                                                  //         return Ok(());
                                                                  //     }
                                                                  //
                                                                  //     self.config
                                                                  //         .current_wallpaper
                                                                  //         .clone_from(&scraped_local_images[0].path);
                                                                  //
                                                                  //     self.set_wallpaper()?;
                                                                  // }
        }

        self.config = config;

        Ok(())
    }
}
