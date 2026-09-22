use crate::backend::{config_get_str, Backend, UploadResult};
use aws_credential_types::Credentials;
use aws_sigv4::http_request::{
    sign, PayloadChecksumKind, SignableBody, SignableRequest, SigningSettings,
    UriPathNormalizationMode,
};
use aws_sigv4::sign::v4;
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};
use std::fs;
use std::time::SystemTime;

pub struct S3Uploader {
    access_key_id: String,
    secret_access_key: String,
    region: String,
    bucket: String,
    endpoint_url: String,
    path: String,
    public_url_base: Option<String>,
    client: Client,
}

impl Backend for S3Uploader {
    fn upload(&self, file_path: &str) -> UploadResult<String> {
        let body = fs::read(file_path)?;

        // 内容哈希取前 8 位作为去重后缀
        let content_hash = {
            let mut hasher = Sha256::new();
            hasher.update(&body);
            hex::encode(hasher.finalize())[..8].to_string()
        };

        let path = std::path::Path::new(file_path);
        let file_stem = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or(file_path);
        let extension = path.extension().and_then(|n| n.to_str()).unwrap_or("");

        let key = if extension.is_empty() {
            format!("{}{}-{}", self.path, file_stem, content_hash)
        } else {
            format!("{}{}-{}.{}", self.path, file_stem, content_hash, extension)
        };

        let (url, host) = self.build_put_url(&key)?;

        let signable = SignableRequest::new(
            "PUT",
            &url,
            std::iter::once(("host", host.as_str())),
            SignableBody::Bytes(&body),
        )
        .map_err(|e| format!("构建签名请求失败: {}", e))?;

        let identity =
            Credentials::new(&self.access_key_id, &self.secret_access_key, None, None, "pic-rs")
                .into();

        let mut settings = SigningSettings::default();
        settings.payload_checksum_kind = PayloadChecksumKind::XAmzSha256;
        settings.uri_path_normalization_mode = UriPathNormalizationMode::Disabled;

        let params = v4::SigningParams::builder()
            .identity(&identity)
            .region(&self.region)
            .name("s3")
            .time(SystemTime::now())
            .settings(settings)
            .build()
            .map_err(|e| format!("签名参数构建失败: {}", e))?
            .into();

        let (instructions, _sig) = sign(signable, &params)
            .map_err(|e| format!("签名失败: {}", e))?
            .into_parts();

        // 应用签名到 http::Request,拿到 header
        let mut http_req = http::Request::builder()
            .method("PUT")
            .uri(&url)
            .body(())
            .map_err(|e| format!("构建请求失败: {}", e))?;
        instructions.apply_to_request_http1x(&mut http_req);

        // 把签名 header 复制到 reqwest 请求
        let mut req = self.client.put(&url).body(body);
        for (name, value) in http_req.headers() {
            req = req.header(name, value);
        }

        let resp = req.send()?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().unwrap_or_default();
            return Err(format!("S3 上传失败: HTTP {} {}", status, text).into());
        }

        Ok(self.public_url(&key))
    }
}

impl S3Uploader {
    pub fn new(
        backend_name: &str,
        cfg: &toml::Table,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let access_key_id =
            config_get_str(backend_name, cfg, "access_key_id", None)?.to_string();
        let secret_access_key =
            config_get_str(backend_name, cfg, "secret_access_key", None)?.to_string();
        let region = config_get_str(backend_name, cfg, "region", None)?.to_string();
        let bucket = config_get_str(backend_name, cfg, "bucket", None)?.to_string();
        let endpoint_url =
            config_get_str(backend_name, cfg, "endpoint_url", Some(""))?.to_string();
        let path = config_get_str(backend_name, cfg, "path", Some(""))?.to_string();
        let public_url_base = {
            let s = config_get_str(backend_name, cfg, "public_url_base", Some(""))?.to_string();
            if s.is_empty() { None } else { Some(s) }
        };

        Ok(Self {
            access_key_id,
            secret_access_key,
            region,
            bucket,
            endpoint_url,
            path,
            public_url_base,
            client: Client::new(),
        })
    }

    /// 计算 PUT 用的 URL 和 Host header(签名需要 host)
    fn build_put_url(&self, key: &str) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(endpoint) = self.non_empty_endpoint() {
            // 自定义端点(RustFS/MinIO/R2/B2):path-style
            let url = format!(
                "{}/{}/{}",
                endpoint.trim_end_matches('/'),
                self.bucket,
                key
            );
            // Host header 要带端口(非标淮端口),用 authority() 而不是 host()
            let host = http::Uri::try_from(&url)?
                .authority()
                .ok_or("endpoint_url 缺少 host")?
                .to_string();
            Ok((url, host))
        } else {
            // AWS 标准:virtual-hosted-style
            let url = format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, key
            );
            let host = format!("{}.s3.{}.amazonaws.com", self.bucket, self.region);
            Ok((url, host))
        }
    }

    fn public_url(&self, key: &str) -> String {
        if let Some(base) = &self.public_url_base {
            format!("{}/{}", base.trim_end_matches('/'), key)
        } else if let Some(endpoint) = self.non_empty_endpoint() {
            format!(
                "{}/{}/{}",
                endpoint.trim_end_matches('/'),
                self.bucket,
                key
            )
        } else {
            format!(
                "https://{}.s3.{}.amazonaws.com/{}",
                self.bucket, self.region, key
            )
        }
    }

    fn non_empty_endpoint(&self) -> Option<&str> {
        if self.endpoint_url.is_empty() {
            None
        } else {
            Some(&self.endpoint_url)
        }
    }
}
