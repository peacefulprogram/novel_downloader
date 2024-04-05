use std::io::Error;

use async_trait::async_trait;
use url::Url;

use crate::novel_trait::{Chapter, NovelDownloader};
use crate::util::{get_html, get_text_of_html_el};

pub struct XBiQuGeDownloader {
    url: Url,
}

impl XBiQuGeDownloader {
    pub fn new(url: &str) -> XBiQuGeDownloader {
        XBiQuGeDownloader {
            url: Url::parse(&url).expect(&format!("非法的链接: {url}"))
        }
    }
}

#[async_trait]
impl NovelDownloader for XBiQuGeDownloader {
    async fn get_title(&self) -> Result<String, Error> {
        let doc = get_html(self.url.as_str()).await?;
        let title = doc.select(&scraper::selector::Selector::parse("#info > h1").unwrap()).next().unwrap().text().next().unwrap();
        Ok(title.to_string())
    }

    async fn get_chapters(&self) -> Result<Vec<Chapter>, Error> {
        let mut list = vec![];
        let doc = get_html(self.url.as_str()).await?;
        let dl = doc.select(&scraper::selector::Selector::parse("#list > dl").unwrap()).next().unwrap();
        let mut dt_count = 0;
        for child in dl.child_elements() {
            match child.value().name() {
                "dt" => dt_count += 1,
                "dd" => if dt_count >= 2 {
                    if let Some(a) = child.child_elements().next() {
                        let href = a.attr("href").unwrap();
                        let name = a.text().next().unwrap().to_string();
                        list.push(Chapter::new(name, href.to_string()))
                    }
                },
                _ => {}
            }
        }
        Ok(list)
    }

    async fn get_content(&self, chap_url: String) -> Result<String, Error> {
        let chap_url = self.url.join(&chap_url).unwrap();
        let doc = get_html(chap_url).await?;
        let mut content = String::new();
        if let Some(container) = doc.select(&scraper::selector::Selector::parse("#content").unwrap()).next() {
            get_text_of_html_el(&container, &mut content);
            content.push_str("\n");
        }
        Ok(content)
    }

    fn concurrency_num(&self) -> usize {
        5
    }
}

