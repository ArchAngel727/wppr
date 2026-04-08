use anyhow::Result;
use scraper::{Html, Selector};
use std::{env, path::PathBuf};

#[allow(dead_code)]
struct App {
    save_dir: PathBuf,
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
                    None => println!("a tag has no href"),
                }
            }
        }
    }

    Some(links)
}

#[tokio::main]
async fn main() -> Result<()> {
    if !cfg!(target_os = "linux") {
        return Ok(());
    }

    let args = &env::args().collect::<Vec<String>>()[1..];
    let url = if !args.is_empty() {
        args[0].to_string()
    } else {
        "https://wallpaper-a-day.com/".to_string()
    };

    let page = download_page(&url).await?;

    let links = scrape_links(&page).await;

    println!("{:#?}", links);
    println!("{:#?}", args);

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
