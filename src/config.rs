use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::paths::CtxPaths;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    pub top_k: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub output_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingConfig {
    pub vector_weight: f32,
    pub keyword_weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxConfig {
    pub version: u32,
    pub defaults: Defaults,
    pub embeddings: EmbeddingsConfig,
    pub ranking: RankingConfig,
    pub data_root: String,
    pub cache_root: String,
}

impl CtxConfig {
    pub fn default_for_paths(paths: &CtxPaths) -> Self {
        Self {
            version: 1,
            defaults: Defaults {
                top_k: 5,
                chunk_size: 1200,
                chunk_overlap: 150,
                output_mode: "text".to_string(),
            },
            embeddings: EmbeddingsConfig {
                provider: "fastembed".to_string(),
                model: "BGESmallENV15".to_string(),
            },
            ranking: RankingConfig {
                vector_weight: 0.7,
                keyword_weight: 0.3,
            },
            data_root: paths.data_root.display().to_string(),
            cache_root: paths.cache_root.display().to_string(),
        }
    }
}

pub async fn load_or_default(paths: &CtxPaths) -> Result<CtxConfig> {
    if !paths.config_path.exists() {
        let config = CtxConfig::default_for_paths(paths);
        save(paths, &config).await?;
        return Ok(config);
    }

    let raw = tokio::fs::read_to_string(&paths.config_path).await?;
    Ok(toml::from_str(&raw)?)
}

pub async fn save(paths: &CtxPaths, config: &CtxConfig) -> Result<()> {
    paths.ensure().await?;
    tokio::fs::write(&paths.config_path, toml::to_string_pretty(config)? + "\n").await?;
    Ok(())
}
