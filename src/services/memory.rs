use anyhow::{anyhow, Result};
use chrono::Utc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    chunking,
    input::{self, InputPayload, SourceType},
    normalize,
    search::SearchResult,
    services::runtime::ServiceRuntime,
    storage::{self, ChunkRecord, DocumentRecord},
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemorySource {
    File { path: String },
    Text { text: String },
    Stdin { text: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryAddRequest {
    pub source: MemorySource,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub chunk_size: Option<usize>,
    pub chunk_overlap: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryAddResponse {
    pub ok: bool,
    pub document_id: String,
    pub chunk_count: usize,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
    pub tags: Vec<String>,
}

pub async fn add(runtime: &ServiceRuntime, request: MemoryAddRequest) -> Result<MemoryAddResponse> {
    let input = resolve_input(request.source).await?;
    let normalized = normalize::normalize_content(&input.content);
    let chunk_size = request
        .chunk_size
        .unwrap_or(runtime.config.defaults.chunk_size);
    let chunk_overlap = request
        .chunk_overlap
        .unwrap_or(runtime.config.defaults.chunk_overlap);
    let chunks = chunking::chunk_text(&normalized, chunk_size, chunk_overlap)?;
    let chunk_texts = chunks
        .iter()
        .map(|chunk| format!("passage: {}", chunk.content))
        .collect::<Vec<_>>();
    let vectors = runtime.provider.embed(&chunk_texts).await?;
    let now = Utc::now().to_rfc3339();
    let document_id = Uuid::new_v4().to_string();
    let title = request.title.or(input.title).unwrap_or_default();

    let document = DocumentRecord {
        id: document_id.clone(),
        source_hash: sha256(&normalized),
        source_path: input
            .source_path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        source_type: format!("{:?}", input.source_type).to_lowercase(),
        title: title.clone(),
        tags: request.tags.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    storage::insert_document(&runtime.db, &document).await?;

    let records = chunks
        .into_iter()
        .zip(vectors.into_iter())
        .map(|(chunk, vector)| ChunkRecord {
            id: Uuid::new_v4().to_string(),
            document_id: document_id.clone(),
            chunk_index: chunk.index as u32,
            content: chunk.content,
            title: title.clone(),
            source_path: document.source_path.clone(),
            tags: request.tags.clone(),
            created_at: now.clone(),
            vector,
        })
        .collect::<Vec<_>>();
    storage::insert_chunks(
        &runtime.db,
        &records,
        runtime.provider.dimension().await? as i32,
    )
    .await?;

    Ok(MemoryAddResponse {
        ok: true,
        document_id,
        chunk_count: records.len(),
        title,
    })
}

pub async fn search(
    runtime: &ServiceRuntime,
    request: MemorySearchRequest,
) -> Result<SearchResult> {
    let query = request.query.trim();
    if query.is_empty() {
        return Err(anyhow!("search query is required"));
    }

    crate::search::run_search(
        &runtime.db,
        runtime.provider.as_ref(),
        &runtime.config,
        query,
        request.top_k.unwrap_or(runtime.config.defaults.top_k),
        &request.tags,
    )
    .await
}

async fn resolve_input(source: MemorySource) -> Result<InputPayload> {
    match source {
        MemorySource::File { path } => input::read_input(Some(path), None, false).await,
        MemorySource::Text { text } => Ok(InputPayload {
            source_type: SourceType::Text,
            source_path: None,
            title: None,
            content: text,
        }),
        MemorySource::Stdin { text } => Ok(InputPayload {
            source_type: SourceType::Stdin,
            source_path: None,
            title: None,
            content: text,
        }),
    }
}

fn sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
