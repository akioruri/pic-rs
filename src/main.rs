use clap::Parser;
use std::process::exit;

mod config;
mod github;

use config::Config;
use github::GitHubUploader;

#[derive(Parser, Debug)]
#[command(author, version, about = "GitHub 图床上传工具")]
struct Args {
    #[arg(required = true)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let config = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let backend = config.backend(&config.default_backend).unwrap_or_else(|| {
        eprintln!("Error: 未找到图床配置");
        exit(1);
    });

    let uploader = match GitHubUploader::new(&config.default_backend, backend) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    let mut urls = Vec::new();
    for file in &args.files {
        match uploader.upload(file) {
            Ok(url) => urls.push(url),
            Err(e) => {
                eprintln!("Error: 上传失败 {}: {}", file, e);
                exit(1);
            }
        }
    }

    for url in &urls {
        println!("{}", url);
    }
}
