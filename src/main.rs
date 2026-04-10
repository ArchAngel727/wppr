mod awww_controller;

use anyhow::{Error, Result, anyhow};
use futures::future::join_all;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
};

use crate::awww_controller::AwwwControlle;

struct App<'a> {
    config: Config,
    args: &'a [String],
}

impl<'a> App<'a> {
    pub fn new(config: Config, args: &'a [String]) -> App<'a> {
        App { config, args }
    }

    pub fn save_file(&self, path: &Path, data: &[u8]) -> Result<()> {
        if !self.config.save_dir.is_dir() {
            fs::create_dir(&self.config.save_dir)?;
        }

        let mut file = File::create(path)?;
        file.write_all(data)?;

        Ok(())
    }
}

pub fn save_file(path: &Path, data: &[u8], save_dir: &Path) -> Result<()> {
    if !save_dir.is_dir() {
        fs::create_dir(save_dir)?;
    }

    let mut file = File::create(path)?;
    file.write_all(data)?;

    Ok(())
}

async fn download_page(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.error_for_status()?.text().await
}

async fn scrape_links(page: &str) -> Result<Vec<String>> {
    let mut links: Vec<String> = vec![];
    let regex = Regex::new(r#"\/d\/(.*?)\/view"#)?;

    let document = Html::parse_document(page);

    let main_selector = Selector::parse("main").unwrap();
    let article_selector = Selector::parse("article.post").unwrap();
    let link_selector = Selector::parse("a").unwrap();

    document.select(&main_selector).for_each(|e| {
        e.select(&article_selector).for_each(|el| {
            el.select(&link_selector)
                .for_each(|link| match link.value().attr("href") {
                    Some(href) => {
                        if href.ends_with(".png") || href.ends_with("sharing") {
                            let link = href.to_string();

                            if link.ends_with("sharing")
                                && let Some(id) = regex.captures(&link)
                            {
                                links.push(format!(
                                    "https://drive.google.com/uc?export=view&id={}",
                                    &id[1]
                                ));
                            } else {
                                links.push(link)
                            }
                        }
                    }
                    _ => println!("a tag has no href"),
                })
        });
    });

    Ok(links)
}

async fn download_image(url: &str) -> Result<Vec<u8>, Error> {
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
    // check if wallpaper exists
    // make call to awww to set wallpaper
    // check if awww exists

    if !app.config.current_wallpaper.exists() {
        return Err(anyhow!("No wallpaper selected"));
    }

    println!("{}", app.config.current_wallpaper.display());
    AwwwControlle::set_wallpaper(&app.config.current_wallpaper)?;

    Ok(())
}

async fn scrape(app: &mut App<'_>) -> Result<()> {
    let url = if app.args.len() > 1 {
        app.args[1].to_string()
    } else {
        "https://wallpaper-a-day.com/".to_string()
    };

    if !url.starts_with("http") {
        return Err(anyhow!("Invalid url"));
    }

    if !app.config.save_dir.exists()
        && let Some(home) = home::home_dir()
    {
        let dir_path = PathBuf::from(format!("{}/Pictures/wppr", home.display()));
        fs::create_dir_all(&dir_path)?;
        app.config.save_dir = dir_path;
    }

    let page = download_page(&url).await?;
    let links: Vec<String> = scrape_links(&page).await?;

    let save_dir = app.config.save_dir.clone();

    let futures: Vec<_> = links
        .into_iter()
        .map(|link| process_image(link, &save_dir))
        .collect();

    let res = join_all(futures).await;

    println!("{:?}", res);
    Ok(())
}

async fn process_image(link: String, save_dir: &Path) -> Result<()> {
    let img = download_image(&link).await?;
    let name: String = Sha256::digest(&link).to_vec()[..8]
        .iter()
        .map(|c| format!("{:02x}", c))
        .collect();

    let mut path = PathBuf::from(save_dir);
    path.push(name);
    path.set_extension("png");

    if path.exists() {
        let local_img = std::fs::read(&path)?;

        if img != local_img {
            save_file(&path, &img, save_dir)?;
        }
    } else {
        save_file(&path, &img, save_dir)?;
    }

    println!("Finished processing image {link}");
    Ok(())
}

async fn menu(app: &mut App<'_>) -> Result<()> {
    if app.args.is_empty() {
        //print_help_menu();
        return Ok(());
    }

    match app.args[0].as_str() {
        "reload" => reload_wallpaper(app)?,
        "pick" => println!("B"),
        "scrape" => scrape(app).await?,
        _ => println!("Aw HELL NAHHH!"),
    };

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Config {
    current_wallpaper: PathBuf,
    current_dir: PathBuf,
    save_dir: PathBuf,
}

fn load_config() -> Result<Config> {
    let path = Path::new("./config.json");

    let default_config = r#"{
        "current_wallpaper": "",
        "current_dir": "",
        "save_dir": ""
    }"#;

    if !path.exists() {
        let mut file = File::create(path)?;
        file.write_all(default_config.as_bytes())?;
        return Ok(serde_json::from_str(default_config)?);
    }

    let s = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(s.as_str())?;

    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    compile_error!("AW HELL NAH I AINT RUNNING ON {}", target_os);

    if !AwwwControlle::is_installed() {
        return Err(anyhow!("awww is not installed"));
    }

    //println!(
    //    "{:#?}",
    //    scrape_links(&download_page("https://wallpaper-a-day.com/").await?)
    //        .await
    //        .unwrap()
    //);

    let config = load_config()?;
    let args = &env::args()
        .collect::<Vec<String>>()
        .iter()
        .map(|arg| arg.to_lowercase())
        .collect::<Vec<String>>()[1..];

    let mut app = App::new(config, args);

    menu(&mut app).await?;

    Ok(())
}

//
//    match Command::new("dwu").args(["--save-dir", "~/.dwu"]).output() {
//        Ok(result) => result.stdout.iter().for_each(|f| {
//            print!("{}", *f as char);
//        }),
//        Err(result) => {
//            println!("{}", result);
//        }
//    }

//async fn scrape_imgur(url: &str) -> Result<Option<String>, Error> {
//    let page = reqwest::get(url).await?.error_for_status()?.text().await?;
//    let document = Html::parse_document(&page);
//    let selector = Selector::parse(".image-placeholder").unwrap();
//
//    if let Some(img) = document.select(&selector).next() {
//        match img.value().attr("href") {
//            Some(href) => Ok(Some(href.to_string())),
//            _ => Ok(None),
//        }
//    } else {
//        Ok(None)
//    }
//}
