use std::io::{Error, ErrorKind};

use async_trait::async_trait;
use scraper::{Html, Selector};
use url::Url;

use crate::novel_trait::{Chapter, NovelDownloader};
use crate::util::{get_html, get_text_of_html_el};

pub struct MbxqDownloader {
    url: Url,
    novel_id: String,
}

impl MbxqDownloader {
    pub fn new(url: &str) -> Self {
        let url = Url::parse(url).expect(&format!("无效的地址{url}"));
        let id = url.path_segments().and_then(|s| {
            let mut p = None;
            for x in s {
                if !x.is_empty() {
                    p = Some(x);
                }
            };
            p
        }).unwrap().to_string();
        return Self {
            url,
            novel_id: id,
        };
    }

    fn get_chapters_from_html(doc: &Html) -> Vec<Chapter> {
        return doc.select(&Selector::parse("body > div.wrap > div.book_last > dl > dd > a").unwrap())
            .map(|el| Chapter::new(el.text().next().unwrap().to_string(), el.attr("href").unwrap().to_string()))
            .collect();
    }

    fn content_next_page_url<'a>(&self, doc: &'a Html) -> Option<&'a str> {
        return doc.select(&Selector::parse("#pb_next").unwrap())
            .next()
            .and_then(|el| el.attr("href"));
    }

    fn find_chapter_id<'a>(&self, url: &'a str) -> Option<&'a str> {
        return url.find(&self.novel_id)
            .and_then(|idx| {
                let s1 = &url[(idx + self.novel_id.len() + 1)..];
                s1.find('/')
                    .or_else(|| s1.rfind('.'))
                    .map(|i| &s1[..i])
            });
    }

    fn is_same_chapter(&self, url1: &str, url2: &str) -> bool {
        let p1 = self.find_chapter_id(url1);
        let p2 = self.find_chapter_id(url2);
        if p1.is_none() || p2.is_none() {
            return false;
        }
        return p1.unwrap() == p2.unwrap();
    }
}

#[async_trait]
impl NovelDownloader for MbxqDownloader {
    async fn get_title(&self) -> Result<String, Error> {
        let doc = get_html(self.url.as_str()).await.unwrap();
        let el = doc.select(&Selector::parse("body > div.wrap > div > div.book_info > div.book_box > dl > dt").unwrap()).next().unwrap().text().next().unwrap();
        Ok(el.to_string())
    }

    async fn get_chapters(&self) -> Result<Vec<Chapter>, Error> {
        let chap_url = self.url.join(&format!("/chapters/{}", self.novel_id)).map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        let page_urls: Vec<Option<Url>>;
        let mut chapters;
        {
            let doc = get_html(chap_url.as_str()).await?;
            page_urls = doc.select(&Selector::parse("body > div.wrap > div.book_clist > div > select > option").unwrap())
                .map(|el| el.attr("value"))
                .map(|va| {
                    if let Some(s) = va {
                        if s.contains("chapters") {
                            return Some(chap_url.join(s).unwrap());
                        }
                    }
                    return None;
                })
                .collect();
            chapters = Self::get_chapters_from_html(&doc);
        }
        if page_urls.len() <= 1 {
            return Ok(chapters);
        }
        for page_url in page_urls {
            if let Some(url) = page_url {
                if let Ok(document) = get_html(url.as_str()).await {
                    chapters.extend(Self::get_chapters_from_html(&document));
                }
            }
        }
        return Ok(chapters);
    }

    async fn get_content(&self, ep_url: String) -> Result<String, Error> {
        let mut url = self.url.join(&ep_url).unwrap();
        let mut content = String::new();
        loop {
            let doc = get_html(url.as_str()).await?;
            for el in doc.select(&Selector::parse("#chaptercontent > .content_detail").unwrap()) {
                get_text_of_html_el(&el, &mut content);
                content.push('\n');
            }
            if let Some(next_page) = self.content_next_page_url(&doc) {
                let next_url = url.join(next_page).unwrap();
                if self.is_same_chapter(next_url.as_str(), url.as_str()) {
                    url = next_url;
                    continue;
                }
            }
            break;
        }
        Ok(content)
    }

    fn concurrency_num(&self) -> usize {
        3
    }
}