use crate::{db_manager::DBManager, local_image::LocalImage, online_image::OnlineImage};

use anyhow::{Result, anyhow};
use chrono::DateTime;
use futures::future::join_all;
use regex::Regex;
use scraper::{Html, Selector};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write,
    path::{Path, PathBuf},
};
use tokio::fs;
use tracing::{error, info};

pub struct Scraper {}

impl Scraper {
    async fn save_file(dir: &Path, name: &Path, data: &[u8]) -> Result<()> {
        fs::create_dir_all(dir)
            .await
            .inspect_err(|e| error!("Failed to create dir while saving file: {e:#}"))?;
        fs::write(name, data)
            .await
            .inspect_err(|e| error!("Failed to write file to disk {e:#}"))?;

        Ok(())
    }

    async fn download_page(url: &str) -> Result<String, reqwest::Error> {
        reqwest::get(url)
            .await
            .inspect_err(|e| error!("Get request failed: {e:#}"))?
            .error_for_status()
            .inspect_err(|e| error!("Get request status code: {e:#}"))?
            .text()
            .await
    }

    async fn download_image(url: &str) -> Result<Vec<u8>> {
        Ok(reqwest::get(url)
            .await
            .inspect_err(|e| error!("Get request failed: {e:#}"))?
            .error_for_status()
            .inspect_err(|e| error!("Get request status code: {e:#}"))?
            .bytes()
            .await
            .inspect_err(|e| error!("Failed to download bytes: {e:#}"))?
            .into_iter()
            .collect())
    }

    async fn process_image(image: &OnlineImage, save_dir: &Path) -> Result<LocalImage> {
        let name: String =
            Sha256::digest(&image.link)[..8]
                .iter()
                .fold(String::new(), |mut acc, slice| {
                    let _ = write!(acc, "{slice:02x}");
                    acc
                });

        let path = PathBuf::from(save_dir).join(&name).with_extension("png");

        if !path.exists() {
            let img = Self::download_image(&image.link).await?;
            Self::save_file(save_dir, &path, &img).await?;
        }

        Ok((path, image.date).into())
    }

    pub fn scrape_links(page: &str) -> Result<Vec<OnlineImage>> {
        let mut links: Vec<OnlineImage> = Vec::new();
        let regex = Regex::new(r"\/d\/(.*?)\/view")?;

        let document = Html::parse_document(page);

        let main_selector = Selector::parse("main").unwrap();
        let article_selector = Selector::parse("article.post").unwrap();
        let link_selector = Selector::parse("a").unwrap();
        let date_selector = Selector::parse("time").unwrap();

        let main = document.select(&main_selector).nth(0).unwrap();

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
                    image.link = href;
                }
            }

            if let Some(date_str) = article
                .select(&date_selector)
                .find_map(|date_element| date_element.value().attr("datetime"))
            {
                match DateTime::parse_from_rfc3339(date_str) {
                    Ok(date) => image.date = date.to_utc(),
                    Err(e) => {
                        let error = anyhow!("Failed to parse date {e:#}");
                        error!("{error}");
                        return Err(error);
                    }
                }
            }

            links.push(image);
        }

        Ok(links)
    }

    pub async fn scrape(save_dir: &Path, url: &str, backstep: u32) -> Result<Vec<LocalImage>> {
        if !url.starts_with("http") {
            return Err(anyhow!("Invalid url"));
        }

        let page = Self::download_page(url).await?;
        let links = &Self::scrape_links(&page)?[backstep as usize..(4 + backstep) as usize];

        let futures: Vec<_> = links
            .iter()
            .map(|link| Self::process_image(link, save_dir))
            .collect();

        let mut res: Vec<LocalImage> = join_all(futures)
            .await
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        res.sort_by_key(|k| k.date);
        res.reverse();

        DBManager::write_local_images_to_db(&res, save_dir).await?;

        Ok(res)
    }

    pub async fn scrape_tags() -> Result<Vec<String>> {
        let page = reqwest::get("https://wallpaper-a-day.com/category/")
            .await
            .inspect_err(|e| error!("Failed to download tags page: {e:#}"))?
            .text()
            .await
            .inspect_err(|e| error!("Failed to get tags page text {e:#}"))?;

        let document = Html::parse_document(&page);
        let mut tags: Vec<String> = vec![];

        let selector = Selector::parse("li.cat-item").unwrap();
        let link_selector = Selector::parse("a").unwrap();

        document.select(&selector).for_each(|li| {
            li.select(&link_selector).for_each(|link| {
                tags.push(
                    link.text()
                        .map(|str| str.to_string().replace([' ', '/'], "-"))
                        .collect(),
                );
            });
        });

        tags.sort();
        tags.dedup();
        tags = tags.iter().map(|str| str.to_ascii_lowercase()).collect();

        Ok(tags)
    }

    pub async fn scrape_loacl_images(
        path: &Path,
        tag: Option<String>,
        backstep: Option<u32>,
    ) -> Result<Vec<LocalImage>> {
        let mut url = String::from("https://wallpaper-a-day.com");
        let tags = Self::scrape_tags().await?;

        if let Some(tag) = tag {
            if let Some(tag) = tags.iter().find(|t| t.starts_with(&tag)) {
                url.push_str("/category/");
                url.push_str(tag);
            } else {
                let e = anyhow!("Tag not found");
                error!("{e}");
                return Err(e);
            }
        }

        Self::scrape(path, &url, backstep.unwrap_or(0)).await
    }
}
