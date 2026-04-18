use std::path::PathBuf;

use anyhow::{Context, Result};
use directories::BaseDirs;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CtxPaths {
    pub data_root: PathBuf,
    pub cache_root: PathBuf,
    pub db_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub models_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub config_path: PathBuf,
}

impl CtxPaths {
    pub fn resolve(data_root: Option<PathBuf>, cache_root: Option<PathBuf>) -> Result<Self> {
        let base_dirs = BaseDirs::new().context("could not resolve platform base directories")?;
        let data_root = data_root
            .or_else(|| env_path("CTX_DATA_DIR"))
            .unwrap_or_else(|| base_dirs.data_dir().join("ctx"));
        let cache_root = cache_root
            .or_else(|| env_path("CTX_CACHE_DIR"))
            .unwrap_or_else(|| base_dirs.cache_dir().join("ctx"));
        Ok(Self::from_roots(data_root, cache_root))
    }

    pub fn from_roots(data_root: impl Into<PathBuf>, cache_root: impl Into<PathBuf>) -> Self {
        let data_root = data_root.into();
        let cache_root = cache_root.into();
        Self {
            db_dir: data_root.join("db"),
            logs_dir: data_root.join("logs"),
            config_path: data_root.join("config.toml"),
            models_dir: cache_root.join("models"),
            tmp_dir: cache_root.join("tmp"),
            data_root,
            cache_root,
        }
    }

    pub async fn ensure(&self) -> Result<()> {
        tokio::fs::create_dir_all(&self.data_root).await?;
        tokio::fs::create_dir_all(&self.db_dir).await?;
        tokio::fs::create_dir_all(&self.logs_dir).await?;
        tokio::fs::create_dir_all(&self.cache_root).await?;
        tokio::fs::create_dir_all(&self.models_dir).await?;
        tokio::fs::create_dir_all(&self.tmp_dir).await?;
        Ok(())
    }
}

fn env_path(name: &str) -> Option<PathBuf> {
    std::env::var_os(name).and_then(|value| {
        if value.is_empty() {
            None
        } else {
            Some(PathBuf::from(value))
        }
    })
}
