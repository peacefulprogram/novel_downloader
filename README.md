# novel_downloader

笔趣阁小说下载器

支持以下网站

- [https://www.xbiquge.bz/](https://www.xbiquge.bz/)
- [http://m.biquxs.com/](http://m.biquxs.com/)

#### 从源码安装
```shell
cargo install --path .
```

#### 使用
```shell
Usage: novel_dl.exe [OPTIONS] <URL>

Arguments:
  <URL>  小说链接

Options:
  -c, --concurrency <CONCURRENCY>  并发数
  -o, --output <OUTPUT>            保存目录
  -h, --help                       Print help
```