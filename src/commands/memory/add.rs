use anyhow::Result;
use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    chunking,
    cli::MemoryAddArgs,
    config,
    embeddings::{local::LocalEmbeddingProvider, provider::EmbeddingProvider},
    input, normalize, output,
    paths::CtxPaths,
    storage::{self, ChunkRecord, DocumentRecord},
};

pub async fn run(args: MemoryAddArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let config = config::load_or_default(&paths).await?;
    let provider = LocalEmbeddingProvider::new(&config.embeddings.model, paths.models_dir.clone(), !args.json);
    let db = storage::init_database(&paths, &provider).await?;

    let input = input::read_input(args.file, args.text, args.stdin).await?;
    let normalized = normalize::normalize_content(&input.content);
    let chunk_size = args.chunk_size.unwrap_or(config.defaults.chunk_size);
    let chunk_overlap = args.chunk_overlap.unwrap_or(config.defaults.chunk_overlap);
    let chunks = chunking::chunk_text(&normalized, chunk_size, chunk_overlap)?;
    let chunk_texts = chunks
        .iter()
        .map(|chunk| format!("passage: {}", chunk.content))
        .collect::<Vec<_>>();
    let vectors = provider.embed(&chunk_texts).await?;
    let now = Utc::now().to_rfc3339();
    let document_id = Uuid::new_v4().to_string();
    let title = args.title.or(input.title).unwrap_or_default();
    let tags = args.tags;

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
        tags: tags.clone(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };
    storage::insert_document(&db, &document).await?;

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
            tags: tags.clone(),
            created_at: now.clone(),
            vector,
        })
        .collect::<Vec<_>>();
    storage::insert_chunks(&db, &records, provider.dimension().await? as i32).await?;

    output::render(
        &serde_json::json!({
            "ok": true,
            "documentId": document_id,
            "chunkCount": records.len(),
            "title": title,
        }),
        args.json,
    )
}

fn sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
