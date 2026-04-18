use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    async fn init(&self) -> Result<()>;
    async fn dimension(&self) -> Result<usize>;
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    async fn embed_query(&self, query: &str) -> Result<Vec<f32>>;
    async fn health_check(&self) -> Result<String>;
}
