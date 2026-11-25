use std::error::Error;

pub type UploadResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub trait Backend: Send + Sync {
    fn upload(&self, file_path: &str) -> UploadResult<String>;
}

pub(crate) fn config_get_str<'a>(
    backend_name: &str,
    cfg: &'a toml::Table,
    key: &str,
    default: Option<&'static str>,
) -> Result<&'a str, Box<dyn Error>> {
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
