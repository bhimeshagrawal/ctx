use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use ctx::{
    config::CtxConfig,
    embeddings::provider::EmbeddingProvider,
    paths::CtxPaths,
    services::{
        memory::{self, MemoryAddRequest, MemorySearchRequest, MemorySource},
        runtime::ServiceRuntime,
    },
    storage,
};
use tempfile::TempDir;

#[tokio::test]
async fn memory_service_adds_and_searches_text() {
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

    let add = memory::add(
        &runtime,
        MemoryAddRequest {
            source: MemorySource::Text {
                text: "ctx mcp design note".to_string(),
            },
            title: Some("design".to_string()),
            tags: vec!["ctx".to_string(), "mcp".to_string()],
            chunk_size: Some(64),
            chunk_overlap: Some(8),
        },
    )
    .await
    .expect("add memory");

    let search = memory::search(
        &runtime,
        MemorySearchRequest {
            query: "mcp design".to_string(),
            top_k: Some(5),
            tags: vec!["ctx".to_string()],
            raw: false,
        },
    )
    .await
    .expect("search memory");

    assert!(add.ok);
    assert!(add.chunk_count >= 1);
    assert!(add.memory_count >= 1);
    assert!(search.count >= 1);
    assert_eq!(search.mode, "memory");
    assert_eq!(search.results[0].kind, "memory");
    assert!(
        search.context_pack.relevant_facts.len()
            + search.context_pack.relevant_procedures.len()
            + search.context_pack.relevant_recent_events.len()
            >= 1
    );
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

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|text| embed_value(text)).collect())
    }

    async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        Ok(embed_value(&format!("query: {query}")))
    }

    async fn health_check(&self) -> Result<String> {
        Ok("ready:fake".to_string())
    }
}

fn embed_value(value: &str) -> Vec<f32> {
    let bytes = value.as_bytes();
    let length = bytes.len() as f32;
    let alpha = bytes
        .iter()
        .filter(|byte| byte.is_ascii_alphabetic())
        .count() as f32;
    let spaces = bytes.iter().filter(|byte| **byte == b' ').count() as f32;
    let mcp = value.to_lowercase().matches("mcp").count() as f32;
    vec![length, alpha, spaces, mcp]
}
