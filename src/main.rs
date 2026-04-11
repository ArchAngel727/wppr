mod awww_controller;

use anyhow::{Error, Result, anyhow};
use chrono::prelude::*;
use futures::future::join_all;
use regex::Regex;
use scraper::{ElementRef, Html, Selector};
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
}

#[derive(PartialOrd, PartialEq, Eq)]
struct Image {
    link: String,
    date: NaiveDate,
}

impl Image {
    fn new() -> Image {
        Image {
            link: String::new(),
            date: NaiveDate::default(),
        }
    }
}

pub fn save_file(dir: &Path, name: &Path, data: &[u8]) -> Result<()> {
    if !dir.is_dir() {
        fs::create_dir(dir)?;
    }

    let mut file = File::create(name)?;
    file.write_all(data)?;

    Ok(())
}

async fn download_page(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.error_for_status()?.text().await
}

async fn scrape_links(page: &str) -> Result<Vec<Image>> {
    let mut links: Vec<Image> = vec![];
    let regex = Regex::new(r#"\/d\/(.*?)\/view"#)?;

    let document = Html::parse_document(page);

    let main_selector = Selector::parse("main").unwrap();
    let article_selector = Selector::parse("article.post").unwrap();
    let link_selector = Selector::parse("a").unwrap();
    let date_selector = Selector::parse("time").unwrap();

    document.select(&main_selector).for_each(|e| {
        e.select(&article_selector).for_each(|article| {
            let mut image = Image::new();

            article
                .select(&link_selector)
                .for_each(|link| match link.value().attr("href") {
                    Some(href) => {
                        if !(href.ends_with(".png") || href.ends_with("sharing")) {
                            return;
                        }

                        let link = href.to_string();

                        if link.ends_with("sharing")
                            && let Some(id) = regex.captures(&link)
                        {
                            image.link =
                                format!("https://drive.google.com/uc?export=view&id={}", &id[1]);
                        } else {
                            image.link = link
                        }
                    }
                    _ => println!("a tag has no href"),
                });

            article.select(&date_selector).for_each(|time: ElementRef| {
                if let Some(date_str) = time.value().attr("datetime") {
                    let date = match DateTime::parse_from_rfc3339(date_str) {
                        Ok(date) => date,
                        Err(e) => {
                            println!("{e}");
                            return;
                        }
                    };

                    image.date = date.date_naive();
                }
            });

            links.push(image);
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
    let links = &scrape_links(&page).await?[..3];

    let save_dir = app.config.save_dir.clone();

    let futures: Vec<_> = links
        .iter()
        .map(|link| process_image(link, &save_dir))
        .collect();

    let mut res: Vec<_> = join_all(futures)
        .await
        .into_iter()
        .filter_map(Result::ok)
        .collect();
    res.sort_by_key(|k| k.1);
    res.reverse();

    println!("{:#?}", res);
    Ok(())
}

async fn process_image(image: &Image, save_dir: &Path) -> Result<(PathBuf, NaiveDate)> {
    let img = download_image(&image.link).await?;
    let name: String = Sha256::digest(&image.link).to_vec()[..8]
        .iter()
        .map(|c| format!("{:02x}", c))
        .collect();

    let mut path = PathBuf::from(save_dir);
    path.push(name);
    path.set_extension("png");

    if path.exists() {
        let local_img = std::fs::read(&path)?;

        if img != local_img {
            save_file(save_dir, &path, &img)?;
        }
    } else {
        println!("Saving file");
        save_file(save_dir, &path, &img)?;
    }

    println!("Finished processing image {}", image.link);
    Ok((path, image.date))
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
