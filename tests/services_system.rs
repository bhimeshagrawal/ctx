use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ctx::{
    config::CtxConfig,
    embeddings::provider::EmbeddingProvider,
    paths::CtxPaths,
    services::{
        runtime::ServiceRuntime,
        system,
        types::UninstallRequest,
    },
    storage,
};
use tempfile::TempDir;

#[tokio::test]
async fn doctor_service_reports_healthy_fake_runtime() {
    let data = TempDir::new().expect("create data tempdir");
    let cache = TempDir::new().expect("create cache tempdir");
    let paths = CtxPaths::from_roots(data.path(), cache.path());
    let config = CtxConfig::default_for_paths(&paths);
    let provider: Arc<dyn EmbeddingProvider> = Arc::new(FakeEmbeddingProvider);
    let db = storage::init_database(&paths, provider.as_ref())
        .await
        .expect("initialize database");
    let runtime = ServiceRuntime {
        paths,
        config,
        provider,
        db,
    };

    let result = system::doctor(&runtime).await.expect("run doctor");

    assert!(result.ok);
    assert_eq!(result.checks.len(), 4);
}

#[tokio::test]
async fn uninstall_service_preserves_data_without_purge() {
    let data = TempDir::new().expect("create data tempdir");
    let cache = TempDir::new().expect("create cache tempdir");
    let paths = CtxPaths::from_roots(data.path(), cache.path());
    paths.ensure().await.expect("create managed directories");

    let result = system::uninstall(
        &paths,
        UninstallRequest {
            purge_data: false,
        },
    )
    .await
    .expect("uninstall managed paths");

    assert!(result.cache_removed);
    assert!(!result.data_removed);
    assert!(result.data_preserved);
}

struct FakeEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for FakeEmbeddingProvider {
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    async fn dimension(&self) -> Result<usize> {
        Ok(4)
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(Vec::new())
    }

    async fn embed_query(&self, _query: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0; 4])
    }

    async fn health_check(&self) -> Result<String> {
        Ok("ready:fake".to_string())
    }
}
