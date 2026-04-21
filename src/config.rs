use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::paths::CtxPaths;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Defaults {
    pub top_k: usize,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub output_mode: String,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            top_k: 5,
            chunk_size: 1200,
            chunk_overlap: 150,
            output_mode: "text".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmbeddingsConfig {
    pub provider: String,
    pub model: String,
}

impl Default for EmbeddingsConfig {
    fn default() -> Self {
        Self {
            provider: "fastembed".to_string(),
            model: "BGESmallENV15".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RankingConfig {
    #[serde(default = "RankingConfig::default_vector_weight")]
    pub vector_weight: f32,
    #[serde(default = "RankingConfig::default_keyword_weight")]
    pub keyword_weight: f32,
    #[serde(default = "RankingConfig::default_title_weight")]
    pub title_weight: f32,
    #[serde(default = "RankingConfig::default_path_weight")]
    pub path_weight: f32,
    #[serde(default = "RankingConfig::default_recency_weight")]
    pub recency_weight: f32,
    #[serde(default = "RankingConfig::default_importance_weight")]
    pub importance_weight: f32,
    #[serde(default = "RankingConfig::default_confidence_weight")]
    pub confidence_weight: f32,
    #[serde(default = "RankingConfig::default_access_weight")]
    pub access_weight: f32,
    #[serde(default = "RankingConfig::default_scope_weight")]
    pub scope_weight: f32,
}

impl RankingConfig {
    pub fn default_vector_weight() -> f32 { 0.45 }
    pub fn default_keyword_weight() -> f32 { 0.2 }
    pub fn default_title_weight() -> f32 { 0.1 }
    pub fn default_path_weight() -> f32 { 0.05 }
    pub fn default_recency_weight() -> f32 { 0.05 }
    pub fn default_importance_weight() -> f32 { 0.05 }
    pub fn default_confidence_weight() -> f32 { 0.05 }
    pub fn default_access_weight() -> f32 { 0.03 }
    pub fn default_scope_weight() -> f32 { 0.02 }
}

impl Default for RankingConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.45,
            keyword_weight: 0.2,
            title_weight: 0.1,
            path_weight: 0.05,
            recency_weight: 0.05,
            importance_weight: 0.05,
            confidence_weight: 0.05,
            access_weight: 0.03,
            scope_weight: 0.02,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RetrievalConfig {
    pub default_mode: String,
    pub max_memories: usize,
    pub max_memory_words: usize,
    pub max_evidence_snippets: usize,
    pub max_evidence_words: usize,
    pub context_word_budget: usize,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            default_mode: "memory".to_string(),
            max_memories: 6,
            max_memory_words: 100,
            max_evidence_snippets: 2,
            max_evidence_words: 60,
            context_word_budget: 480,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CtxConfig {
    #[serde(default = "default_config_version")]
    pub version: u32,
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub embeddings: EmbeddingsConfig,
    #[serde(default)]
    pub ranking: RankingConfig,
    #[serde(default)]
    pub retrieval: RetrievalConfig,
    pub data_root: String,
    pub cache_root: String,
}

impl CtxConfig {
    pub fn default_for_paths(paths: &CtxPaths) -> Self {
        Self {
            version: default_config_version(),
            defaults: Defaults::default(),
            embeddings: EmbeddingsConfig::default(),
            ranking: RankingConfig::default(),
            retrieval: RetrievalConfig::default(),
            data_root: paths.data_root.display().to_string(),
            cache_root: paths.cache_root.display().to_string(),
        }
    }
}

fn default_config_version() -> u32 {
    2
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
