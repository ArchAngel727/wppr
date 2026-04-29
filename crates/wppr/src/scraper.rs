use crate::{app::App, local_image::LocalImage, online_image::OnlineImage};

use anyhow::{Result, anyhow};
use chrono::DateTime;
use futures::future::join_all;
use regex::Regex;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct Scraper {}

impl Scraper {
    async fn save_file(dir: &Path, name: &Path, data: &[u8]) -> Result<()> {
        fs::create_dir_all(dir).await?;
        fs::write(name, data).await?;

        Ok(())
    }

    async fn download_page(url: &str) -> Result<String, reqwest::Error> {
        reqwest::get(url).await?.error_for_status()?.text().await
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

    async fn process_image(image: &OnlineImage, save_dir: &Path) -> Result<LocalImage> {
        let name: String = Sha256::digest(&image.link).to_vec()[..8]
            .iter()
            .map(|c| format!("{:02x}", c))
            .collect();

        let path = PathBuf::from(save_dir).join(name).with_extension("png");

        if !path.exists() {
            let img = Scraper::download_image(&image.link).await?;
            Scraper::save_file(save_dir, &path, &img).await?;
        }

        Ok((path, image.date).into())
    }

    pub async fn scrape_links(page: &str) -> Result<Vec<OnlineImage>> {
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

    pub async fn scrape(app: &mut App<'_>, url: &str, backstep: u32) -> Result<Vec<LocalImage>> {
        if !url.starts_with("http") {
            return Err(anyhow!("Invalid url"));
        }

        let save_dir = app.config.save_dir.clone();
        let page = Scraper::download_page(url).await?;
        let links =
            &Scraper::scrape_links(&page).await?[backstep as usize..(4 + backstep) as usize];

        let futures: Vec<_> = links
            .iter()
            .map(|link| Scraper::process_image(link, &save_dir))
            .collect();

        let mut res: Vec<LocalImage> = join_all(futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        res.sort_by_key(|k| k.date);
        res.reverse();

        Ok(res)
    }

    pub async fn scrape_tags() -> Result<Vec<String>> {
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
}
