use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs;

pub struct GitHubUploader {
    token: String,
    repo: String,
    branch: String,
    path: String,
    client: Client,
}

impl GitHubUploader {
    pub fn new(
        backend_name: &str,
        cfg: &toml::Table,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        fn get<'a>(
            backend_name: &str,
            cfg: &'a toml::Table,
            key: &str,
            default: Option<&'static str>,
        ) -> Result<&'a str, Box<dyn std::error::Error>> {
            match cfg.get(key) {
                None => default.ok_or_else(|| {
                    format!("后端 `{}` 缺少必需的 `{}` 字段", backend_name, key).into()
                }),
                Some(v) => v.as_str().ok_or_else(|| {
                    format!(
                        "后端 `{}` 的 `{}` 字段类型错误: 应为字符串",
                        backend_name, key
                    )
                    .into()
                }),
            }
        }

        let token = get(backend_name, cfg, "token", None)?.to_string();
        let repo = get(backend_name, cfg, "repo", None)?.to_string();
        let branch = get(backend_name, cfg, "branch", Some("main"))?.to_string();
        let path = get(backend_name, cfg, "path", Some(""))?.to_string();

        Ok(Self {
            token,
            repo,
            branch,
            path,
            client: Client::new(),
        })
    }

    fn api_url(&self, file_name: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/contents/{}{}",
            self.repo, self.path, file_name
        )
    }

    fn raw_url(&self, file_name: &str) -> String {
        format!(
            "https://raw.githubusercontent.com/{}/{}/{}{}",
            self.repo, self.branch, self.path, file_name
        )
    }

    pub fn upload(
        &self,
        file_path: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let path = std::path::Path::new(file_path);
        let file_stem = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);
        let extension = path.extension().and_then(|n| n.to_str()).unwrap_or("");

        let content = fs::read(file_path)?;
        let content_base64 = BASE64.encode(&content);

        // 生成内容哈希，取前8位
        let content_hash = {
            let mut hasher = Sha256::new();
            hasher.update(&content);
            hex::encode(hasher.finalize())[..8].to_string()
        };

        // 新文件名：原名-内容hash.扩展名
        let new_file_name = if extension.is_empty() {
            format!("{}-{}", file_stem, content_hash)
        } else {
            format!("{}-{}.{}", file_stem, content_hash, extension)
        };

        let url = self.api_url(&new_file_name);

        // 检查是否已存在
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "pic-rs")
            .send()?;

        let sha_to_use: Option<String> = if resp.status() == 200 {
            let body = resp.text()?;
            serde_json::from_str::<serde_json::Value>(&body)
                .ok()
                .and_then(|json| json.get("sha")?.as_str()?.to_string().into())
        } else {
            None
        };

        #[derive(serde::Serialize)]
        struct Request<'a> {
            message: &'a str,
            content: &'a str,
            branch: &'a str,
            sha: Option<&'a str>,
        }

        let req = Request {
            message: &format!("Upload {}", new_file_name),
            content: &content_base64,
            branch: &self.branch,
            sha: sha_to_use.as_deref(),
        };

        let resp = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("User-Agent", "pic-rs")
            .header("Content-Type", "application/json")
            .json(&req)
            .send()?;

        if !resp.status().is_success() {
            let body = resp.text()?;
            return Err(format!("GitHub API 错误: {}", body).into());
        }

        Ok(self.raw_url(&new_file_name))
    }
}
