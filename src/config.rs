use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default_backend: String,
    pub backends: toml::Table,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Err(format!("配置文件不存在: {:?}", path).into());
        }

        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;

        Ok(config)
    }

    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let candidates = [
            dirs::home_dir().map(|p| p.join(".config/pic-rs/config.toml")),
            Some(std::env::current_dir()?.join(".pic-rs.toml")),
            Some(std::env::current_dir()?.join("pic-rs.toml")),
        ];

        for path in candidates.into_iter().flatten() {
            if path.exists() {
                return Ok(path);
            }
        }

        Err("未找到配置文件，请创建 .pic-rs.toml".into())
    }

    pub fn backend(&self, name: &str) -> Option<&toml::Table> {
        self.backends.get(name).and_then(|v| v.as_table())
    }
}
