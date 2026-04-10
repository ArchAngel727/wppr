use anyhow::{Error, Result};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    path::{Path, PathBuf},
};

struct App<'a> {
    config: Config,
    args: &'a [String],
}

impl<'a> App<'a> {
    pub fn new(config: Config, args: &'a [String]) -> App<'a> {
        App { config, args }
    }
}

async fn download_page(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.error_for_status()?.text().await
}

async fn scrape_links(page: &str) -> Option<Vec<String>> {
    let mut links: Vec<String> = vec![];

    let document = Html::parse_document(page);
    let main_selector = Selector::parse("main").unwrap();
    let touhou_selector = Selector::parse(".tag-touhou").unwrap();
    let link_selector = Selector::parse("a").unwrap();

    if let Some(main) = document.select(&main_selector).next() {
        for el in main.select(&touhou_selector) {
            if let Some(a) = el.select(&link_selector).next() {
                match a.value().attr("href") {
                    Some(href) => links.push(href.to_string()),
                    _ => println!("a tag has no href"),
                }
            }
        }
    }

    Some(links)
}

fn get_image_hash(image: &[u8]) -> Vec<u8> {
    Sha256::digest(image).into_iter().collect::<Vec<u8>>()
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

fn save_file(name: &Path, data: &[u8]) -> Result<()> {
    if !Path::new("./Downloads/").is_dir() {
        let _ = fs::create_dir("./Downloads");
    }

    let mut file = File::create(name)?;
    let _ = file.write_all(data);

    Ok(())
}

fn reload_wallpaper(app: &App) {
    // check if wallpaper exists
    // make call to awww to set wallpaper
    // check if awww exists

    if !app.config.current_wallpaper.exists() {
        return;
    }

    println!("{}", app.config.current_wallpaper.display());
}

async fn scrape(app: &mut App<'_>) -> Result<()> {
    //let url = if !app.args.is_empty() {
    //    app.args[0].to_string()
    //} else {
    //    "https://wallpaper-a-day.com/".to_string()
    //};

    let url = "https://wallpaper-a-day.com/".to_string();

    if !url.starts_with("http") {
        return Ok(());
    }

    let page = download_page(&url).await?;
    let links: Vec<String> = scrape_links(&page).await.expect("ASDFF");

    if !app.config.save_dir.exists()
        && let Some(home) = home::home_dir()
    {
        println!("{}", home.display());
        let dir_path = PathBuf::from(format!("{}/Pictures/wppr", home.display()));
        println!("{}", dir_path.display());
        fs::create_dir_all(&dir_path)?;
        app.config.save_dir = dir_path;
    }

    let save_dir = app.config.save_dir.clone();
    println!("Save dir: {}", save_dir.display());

    for link in links {
        let img = download_image(&link).await?;

        //let hash = get_image_hash(&img);

        if let Some(name) = link.split("/").last() {
            let file_path = format!("{}/{}", save_dir.display(), name);
            let path = Path::new(&file_path);

            if path.exists() {
                let local_img = std::fs::read(path)?;
                if img == local_img {
                    println!("{}", img == local_img);
                    continue;
                }
            }

            save_file(path, &img)?;
        }
    }

    Ok(())
}

async fn menu(app: &mut App<'_>) -> Result<()> {
    if app.args.is_empty() {
        //print_help_menu();
        return Ok(());
    }

    match app.args[0].as_str() {
        "reload" => reload_wallpaper(app),
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
    if !cfg!(target_os = "linux") {
        return Ok(());
    }

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

//    let a = Command::new("dwu")
//        .arg("--help")
//        .output()
//        .expect("dwu error!");
//
//    a.stdout.iter().for_each(|f| {
//        print!("{}", *f as char);
//    });
//    println!("{:?}", a.stdout);
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
