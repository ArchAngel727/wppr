mod app;
mod cli;
mod config;
mod local_image;
mod online_image;

use crate::cli::Cli;
use crate::config::Config;
use crate::online_image::OnlineImage;
use crate::{app::App, local_image::LocalImage};

use anyhow::{Result, anyhow};
use awww::AwwwController;
use chrono::DateTime;
use clap::Parser;
use futures::future::join_all;
use matugen::MatugenController;
use regex::Regex;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::{
    fs::{self as stdfs, File},
    io::prelude::*,
    path::{Path, PathBuf},
};
use tokio::fs;

async fn save_file(dir: &Path, name: &Path, data: &[u8]) -> Result<()> {
    tokio::fs::create_dir_all(dir).await?;
    tokio::fs::write(name, data).await?;

    Ok(())
}

async fn download_page(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.error_for_status()?.text().await
}

async fn scrape_links(page: &str) -> Result<Vec<OnlineImage>> {
    let mut links: Vec<OnlineImage> = vec![];
    let regex = Regex::new(r#"\/d\/(.*?)\/view"#)?;

    let document = Html::parse_document(page);

    let main_selector = Selector::parse("main").unwrap();
    let article_selector = Selector::parse("article.post").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let date_selector = Selector::parse("time").unwrap();

    let main = document.select(&main_selector).collect::<Vec<_>>()[0];

    for article in main.select(&article_selector) {
        let mut image = OnlineImage::new();

        if let Some(href) = article
            .select(&link_selector)
            .filter_map(|link| link.value().attr("href").map(str::to_string))
            .find(|href| href.ends_with(".png") || href.ends_with("sharing"))
        {
            if href.ends_with("sharing")
                && let Some(id) = regex.captures(&href)
            {
                image.link = format!("https://drive.google.com/uc?export=view&id={}", &id[1]);
            } else {
                image.link = href
            }
        }

        if let Some(date_str) = article
            .select(&date_selector)
            .filter_map(|date_element| date_element.value().attr("datetime"))
            .next()
        {
            match DateTime::parse_from_rfc3339(date_str) {
                Ok(date) => image.date = date,
                Err(e) => {
                    return Err(anyhow!("Failed to parse date {e}"));
                }
            };
        }

        links.push(image);
    }

    Ok(links)
}

async fn download_image(url: &str) -> Result<Vec<u8>> {
    Ok(reqwest::get(url)
        .await?
        .error_for_status()?
        .bytes()
        .await
        .unwrap()
        .into_iter()
        .collect())
}

fn reload_wallpaper(app: &App) -> Result<()> {
    if !app.config.current_wallpaper.exists() {
        println!("{}", app.config.current_wallpaper.display());
        return Err(anyhow!("No wallpaper selected"));
    }

    println!("{}", app.config.current_wallpaper.display());
    AwwwController::set_wallpaper(&app.config.current_wallpaper)?;
    MatugenController::update_colors(&app.config.current_wallpaper)?;

    Ok(())
}

async fn scrape(app: &mut App<'_>, url: &str, backstep: u32) -> Result<()> {
    if !url.starts_with("http") {
        return Err(anyhow!("Invalid url"));
    }

    if !app.config.save_dir.exists()
        && let Some(home) = home::home_dir()
    {
        let dir_path = PathBuf::from(format!("{}/Pictures/wppr", home.display()));
        fs::create_dir_all(&dir_path).await?;
        app.config.save_dir = dir_path;
    }

    let save_dir = app.config.save_dir.clone();
    let page = download_page(url).await?;
    let links = &scrape_links(&page).await?[backstep as usize..4];

    let futures: Vec<_> = links
        .iter()
        .map(|link| process_image(link, &save_dir))
        .collect();

    let mut res: Vec<LocalImage> = join_all(futures)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    res.sort_by_key(|k| k.date);
    res.reverse();

    res.iter().for_each(|img| println!("{}", img));

    app.config.current_wallpaper = res[0].path.clone();
    AwwwController::set_wallpaper(&app.config.current_wallpaper)?;
    MatugenController::update_colors(&app.config.current_wallpaper)?;
    save_config(app)?;

    Ok(())
}

async fn process_image(image: &OnlineImage, save_dir: &Path) -> Result<LocalImage> {
    let name: String = Sha256::digest(&image.link).to_vec()[..8]
        .iter()
        .map(|c| format!("{:02x}", c))
        .collect();

    let path = PathBuf::from(save_dir).join(name).with_extension("png");

    if !path.exists() {
        let img = download_image(&image.link).await?;
        save_file(save_dir, &path, &img).await?;
    }

    Ok((path, image.date).into())
}

async fn scrape_tags() -> Result<Vec<String>> {
    let page = reqwest::get("https://wallpaper-a-day.com/category/")
        .await?
        .text()
        .await?;

    let document = Html::parse_document(&page);
    let mut tags: Vec<String> = vec![];

    let selector = Selector::parse("li.cat-item").unwrap();
    let link_selector = Selector::parse("a").unwrap();

    document.select(&selector).for_each(|li| {
        li.select(&link_selector).for_each(|link| {
            tags.push(
                link.text()
                    .map(|str| str.to_string().replace(" ", "-").replace("/", "-"))
                    .collect(),
            );
        });
    });

    tags.sort();
    tags.dedup();
    tags = tags.iter().map(|str| str.to_ascii_lowercase()).collect();

    Ok(tags)
}

fn save_config(app: &App) -> Result<()> {
    if !app.config_path.exists()
        && let Some(dir) = app.config_path.parent()
    {
        stdfs::create_dir_all(dir)?;
    }

    let mut file = File::create(app.config_path)?;
    file.write_all(&serde_json::to_vec_pretty(&app.config)?)?;

    Ok(())
}

fn load_config(path: &Path) -> Result<Config> {
    let default_config = r#"{
        "current_wallpaper": "",
        "current_dir": "",
        "save_dir": ""
    }"#;

    if let Some(dir) = path.parent()
        && !dir.exists()
    {
        stdfs::create_dir_all(dir)?;
    }

    if !path.exists() {
        let mut file = File::create(path)?;
        file.write_all(default_config.as_bytes())?;

        Ok(serde_json::from_str(default_config)?)
    } else {
        Ok(serde_json::from_str(stdfs::read_to_string(path)?.as_str())?)
    }
}

async fn menu(app: &mut App<'_>) -> Result<()> {
    let mut url = "https://wallpaper-a-day.com".to_string();

    match &app.args.command {
        cli::Commands::Reload => reload_wallpaper(app)?,
        cli::Commands::Pick => todo!("pick"),
        cli::Commands::Scrape { tag, backstep } => {
            let tags = scrape_tags().await?;

            if let Some(tag) = tag {
                match tags.iter().find(|t| t.starts_with(tag)) {
                    Some(tag) => {
                        url.push_str("/category/");
                        url.push_str(tag);
                    }
                    None => todo!(),
                }
            }

            scrape(app, &url, backstep.unwrap_or(0)).await?;
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    compile_error!("AW HELL NAH I AINT RUNNING ON {}", target_os);

    let cli = Cli::parse();

    if !AwwwController::is_installed() {
        return Err(anyhow!("awww is not installed"));
    }

    if !MatugenController::is_installed() {
        return Err(anyhow!("matugen is not installed"));
    }

    let config_path = PathBuf::from(if let Some(home) = home::home_dir() {
        format!("{}/.config/wppr/config.json", home.display())
    } else {
        return Err(anyhow!("Could not find home dir"));
    });

    let config = load_config(&config_path)?;
    let mut app = App::new(&config_path, config, cli);

    menu(&mut app).await?;

    Ok(())
}
