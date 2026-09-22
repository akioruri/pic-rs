use clap::Parser;
use std::process::exit;

mod backend;
mod config;
mod github;
mod s3;

use backend::Backend;
use config::Config;
use github::GitHubUploader;
use s3::S3Uploader;

#[derive(Parser, Debug)]
#[command(author, version, about = "图片上传工具")]
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
        eprintln!("Error: 未找到图床配置 `{}`", config.default_backend);
        exit(1);
    });

    let uploader: Box<dyn Backend> = match config.default_backend.as_str() {
        "github" => match GitHubUploader::new(&config.default_backend, backend) {
            Ok(u) => Box::new(u),
            Err(e) => {
                eprintln!("Error: {}", e);
                exit(1);
            }
        },
        "s3" => match S3Uploader::new(&config.default_backend, backend) {
            Ok(u) => Box::new(u),
            Err(e) => {
                eprintln!("Error: {}", e);
                exit(1);
            }
        },
        other => {
            eprintln!(
                "Error: 不支持的后端 `{}` (当前支持: github, s3)",
                other
            );
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
