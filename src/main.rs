use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::sync::{Arc, Mutex};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use tokio;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use novel_trait::NovelDownloader;

use crate::mbqg::MbxqDownloader;
use crate::xbqg::XBiQuGeDownloader;

mod novel_trait;
mod xbqg;
mod mbqg;
mod util;


#[derive(Parser, Debug)]
#[command(about)]
struct CliArg {
    /// 并发数
    #[arg(short, long)]
    concurrency: Option<usize>,

    /// 保存目录
    #[arg(short, long)]
    output: Option<String>,

    /// 小说链接
    url: String,
}

#[tokio::main]
async fn main() {
    let args = CliArg::parse();
    let d = Arc::new(Box::into_pin(get_downloader(&args.url).expect("非法地址")));
    let title = d.get_title().await.expect("未获取到标题");
    let chapters = d.get_chapters().await.expect("未获取到章节");
    if chapters.is_empty() {
        eprintln!("未获取到章节");
        return;
    }
    println!("开始下载 {title}");
    let save_dir = args.output.unwrap_or_else(|| std::env::current_dir().unwrap().to_str().unwrap().to_string());
    let save_dir = PathBuf::from(save_dir);
    if !save_dir.exists() {
        fs::create_dir_all(&save_dir).expect("创建保存目录失败")
    }
    let mut js = JoinSet::new();
    let chapter_paths: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(vec![]));
    // 取消后删除临时文件
    {
        let paths = chapter_paths.clone();
        let _ = ctrlc::set_handler(move || {
            let guard = paths.lock().unwrap();
            for p in guard.iter() {
                if p.exists() {
                    let _ = fs::remove_file(p);
                }
            }
            exit(0);
        });
    }
    let sm = Arc::new(Semaphore::new(args.concurrency.filter(|i| i.ge(&0)).unwrap_or_else(|| d.concurrency_num())));
    let progress_bar = ProgressBar::new(chapters.len() as u64);
    progress_bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})").unwrap());
    progress_bar.tick();
    let progress_bar = Arc::new(Mutex::new(progress_bar));
    let temp_dir = std::env::temp_dir();
    for chapter in &chapters {
        let downloader = d.clone();
        let ep_url = chapter.url.clone();
        let chap_name = chapter.name.clone();
        let save_path = temp_dir.join(Path::new(&uuid::Uuid::new_v4().to_string()));
        chapter_paths.lock().unwrap().push(save_path.clone());
        let sm = sm.clone();
        let pb = progress_bar.clone();
        js.spawn(async move {
            let _permit = sm.acquire().await.unwrap();
            if let Ok(content) = downloader.get_content(ep_url).await {
                let mut file = File::create(save_path).unwrap();
                file.write(chap_name.as_bytes()).unwrap();
                file.write("\n".as_bytes()).unwrap();
                file.write(content.as_bytes()).unwrap();
            };
            pb.lock().unwrap().inc(1)
        });
    };
    while let Some(_) = js.join_next().await {};
    progress_bar.lock().unwrap().finish();
    combine_novel(&title, &save_dir, chapter_paths.lock().unwrap().as_ref());
}

fn get_downloader(url: &str) -> Option<Box<dyn NovelDownloader>> {
    if url.contains("www.xbiquge.bz") {
        Some(Box::new(XBiQuGeDownloader::new(url)))
    } else if url.contains("m.biquxs.com") {
        Some(Box::new(MbxqDownloader::new(url)))
    } else {
        None
    }
}

fn combine_novel(novel_name: &str, dir: &PathBuf, chap_paths: &Vec<PathBuf>) {
    let save_path = dir.join(novel_name.to_owned() + ".txt");
    let mut file = File::create(&save_path).unwrap();
    file.write(novel_name.as_bytes()).unwrap();
    for path in chap_paths {
        if !path.exists() {
            continue;
        }
        file.write("\n".as_bytes()).unwrap();
        file.write(fs::read(path).unwrap().as_slice()).unwrap();
        let _ = fs::remove_file(path);
    }
    let abs_path = save_path.as_path().canonicalize().unwrap();
    let mut file_path = abs_path.to_str().unwrap();
    if file_path.starts_with(r"\\?\") {
        file_path = &file_path[4..];
    }
    println!("下载完成,已保存到 {}", file_path);
}


