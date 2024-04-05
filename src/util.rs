use std::io::{Error, ErrorKind};

use reqwest::IntoUrl;
use scraper::ElementRef;

pub async fn get_html(url: impl IntoUrl) -> Result<scraper::Html, Error> {
    let text = reqwest::get(url)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?
        .text()
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
    Ok(scraper::Html::parse_document(&text))
}

pub fn get_text_of_html_el(el: &ElementRef, dest: &mut String) {
    let mut first_line = true;
    for line in el.text() {
        if line.trim().is_empty() {
            continue;
        }
        dest.push_str(&line.replace("\u{A0}", " "));
        if first_line {
            dest.push('\n');
            first_line = false;
        };
    }
}