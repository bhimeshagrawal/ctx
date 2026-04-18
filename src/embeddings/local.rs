use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use tokio::sync::Mutex;

use crate::embeddings::provider::EmbeddingProvider;

pub struct LocalEmbeddingProvider {
    model_name: String,
    cache_dir: PathBuf,
    show_download_progress: bool,
    model: Arc<Mutex<Option<TextEmbedding>>>,
}

impl LocalEmbeddingProvider {
    pub fn new(model_name: &str, cache_dir: PathBuf, show_download_progress: bool) -> Self {
        Self {
            model_name: model_name.to_string(),
            cache_dir,
            show_download_progress,
            model: Arc::new(Mutex::new(None)),
        }
    }

    fn model_enum(&self) -> EmbeddingModel {
        match self.model_name.as_str() {
            "BGESmallENV15" => EmbeddingModel::BGESmallENV15,
            _ => EmbeddingModel::BGESmallENV15,
        }
    }

    async fn with_model<R>(&self, f: impl FnOnce(&mut TextEmbedding) -> Result<R>) -> Result<R> {
        let mut guard = self.model.lock().await;
        if guard.is_none() {
            let model = TextEmbedding::try_new(
                InitOptions::new(self.model_enum())
                    .with_cache_dir(self.cache_dir.clone())
                    .with_show_download_progress(self.show_download_progress),
            )?;
            *guard = Some(model);
        }
        let model = guard.as_mut().ok_or_else(|| anyhow!("embedding model was not initialized"))?;
        f(model)
    }
}

#[async_trait]
impl EmbeddingProvider for LocalEmbeddingProvider {
    async fn init(&self) -> Result<()> {
        self.with_model(|_| Ok(())).await
    }

    async fn dimension(&self) -> Result<usize> {
        Ok(TextEmbedding::list_supported_models()
            .into_iter()
            .find(|entry| entry.model == self.model_enum())
            .map(|entry| entry.dim)
            .unwrap_or(384))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.with_model(|model| Ok(model.embed(texts, None)?)).await
    }

    async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let value = format!("query: {query}");
        let vectors = self.embed(&[value]).await?;
        vectors.into_iter().next().ok_or_else(|| anyhow!("no query embedding returned"))
    }

    async fn health_check(&self) -> Result<String> {
        self.init().await?;
        Ok(format!("ready:{}", self.model_name))
    }
}
