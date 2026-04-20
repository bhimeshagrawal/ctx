use std::{path::PathBuf, sync::Arc};

use anyhow::Result;

use crate::{
    config::{self, CtxConfig},
    embeddings::{local::LocalEmbeddingProvider, provider::EmbeddingProvider},
    paths::CtxPaths,
    storage::{self, CtxDatabase},
};

pub struct ServiceRuntime {
    pub paths: CtxPaths,
    pub config: CtxConfig,
    pub provider: Arc<dyn EmbeddingProvider>,
    pub db: CtxDatabase,
}

impl ServiceRuntime {
    pub async fn bootstrap(
        data_root: Option<PathBuf>,
        cache_root: Option<PathBuf>,
        verbose_embeddings: bool,
    ) -> Result<Self> {
        let paths = CtxPaths::resolve(data_root, cache_root)?;
        let config = config::load_or_default(&paths).await?;
        let provider: Arc<dyn EmbeddingProvider> = Arc::new(LocalEmbeddingProvider::new(
            &config.embeddings.model,
            paths.models_dir.clone(),
            verbose_embeddings,
        ));
        let db = storage::init_database(&paths, provider.as_ref()).await?;

        Ok(Self {
            paths,
            config,
            provider,
            db,
        })
    }
}
