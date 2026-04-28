use anyhow::{Result, anyhow};
use awww::AwwwController;
use matugen::MatugenController;
use std::path::Path;

use crate::cli::Cli;
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

    pub fn reload_wallpaper(&self) -> Result<()> {
        if !self.config.current_wallpaper.exists() {
            println!("{}", self.config.current_wallpaper.display());
            return Err(anyhow!("No wallpaper selected"));
        }

        println!("{}", self.config.current_wallpaper.display());
        AwwwController::set_wallpaper(&self.config.current_wallpaper)?;
        MatugenController::update_colors(&self.config.current_wallpaper)?;

        Ok(())
    }

    pub async fn menu(&mut self) -> Result<()> {
        let mut url = "https://wallpaper-a-day.com".to_string();

        match &self.args.command {
            cli::Commands::Reload => self.reload_wallpaper()?,
            cli::Commands::Pick => todo!("pick"),
            cli::Commands::Scrape { tag, backstep } => {
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

                Scraper::scrape(self, &url, backstep.unwrap_or(0)).await?;
            }
        };

        Ok(())
    }
}
