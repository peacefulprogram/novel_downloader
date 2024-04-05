use std::io::Error;

use async_trait::async_trait;

#[derive(Clone)]
pub struct Chapter {
    pub url: String,
    pub name: String,
}

impl Chapter {
    pub fn new(chap_name: String, chap_url: String) -> Chapter {
        Chapter {
            url: chap_url,
            name: chap_name,
        }
    }
}

#[async_trait]
pub trait NovelDownloader: Sync + Send {
    async fn get_title(&self) -> Result<String, Error>;

    async fn get_chapters(&self) -> Result<Vec<Chapter>, Error>;

    async fn get_content(&self, ep_url: String) -> Result<String, Error>;

    fn concurrency_num(&self) -> usize;
}
