use std::collections::HashSet;

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
    storage::{self, ChunkRecord, DocumentRecord, MemoryRecord, MemoryRelationRecord},
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
    pub memory_count: usize,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub raw: bool,
}

#[derive(Debug, Clone)]
struct DraftMemory {
    memory_type: String,
    scope: String,
    title: String,
    content: String,
    summary: String,
    document_id: String,
    source_path: String,
    source_refs: Vec<String>,
    tags: Vec<String>,
    entity_refs: Vec<String>,
    confidence: f32,
    importance: f32,
    status: String,
    created_at: String,
    updated_at: String,
    occurred_at: String,
    normalized_key: String,
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

    let drafts = extract_memories(&document, &records, &request.tags, &now);
    let memory_records = materialize_memories(runtime, drafts).await?;
    storage::insert_memories(
        &runtime.db,
        &memory_records,
        runtime.provider.dimension().await? as i32,
    )
    .await?;
    let relations = build_relations(&memory_records, &now);
    storage::insert_memory_relations(&runtime.db, &relations).await?;

    Ok(MemoryAddResponse {
        ok: true,
        document_id,
        chunk_count: records.len(),
        memory_count: memory_records.len(),
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
        request.raw,
    )
    .await
}

async fn materialize_memories(
    runtime: &ServiceRuntime,
    drafts: Vec<DraftMemory>,
) -> Result<Vec<MemoryRecord>> {
    if drafts.is_empty() {
        return Ok(Vec::new());
    }

    let existing = storage::list_memories(&runtime.db).await?;
    let mut seen = existing
        .into_iter()
        .map(|candidate| normalize::normalize_content(&candidate.content))
        .collect::<HashSet<_>>();
    let memory_inputs = drafts
        .iter()
        .map(|draft| format!("memory: {} {}", draft.title, draft.content))
        .collect::<Vec<_>>();
    let vectors = runtime.provider.embed(&memory_inputs).await?;
    let mut records = Vec::new();

    for (draft, vector) in drafts.into_iter().zip(vectors.into_iter()) {
        if seen.contains(&draft.normalized_key) {
            continue;
        }
        let nearest = storage::vector_search_memories(&runtime.db, &vector, 1).await?;
        if nearest.iter().any(|candidate| {
            candidate.vector_score >= 0.98
                || normalize::normalize_content(&candidate.content) == draft.normalized_key
        }) {
            continue;
        }

        seen.insert(draft.normalized_key);
        records.push(MemoryRecord {
            id: Uuid::new_v4().to_string(),
            memory_type: draft.memory_type,
            scope: draft.scope,
            title: draft.title,
            content: draft.content,
            summary: draft.summary,
            document_id: draft.document_id,
            source_path: draft.source_path,
            source_refs: draft.source_refs,
            tags: draft.tags,
            entity_refs: draft.entity_refs,
            confidence: draft.confidence,
            importance: draft.importance,
            status: draft.status,
            created_at: draft.created_at.clone(),
            updated_at: draft.updated_at,
            occurred_at: draft.occurred_at,
            last_accessed_at: String::new(),
            access_count: 0,
            vector,
        });
    }

    Ok(records)
}

fn extract_memories(
    document: &DocumentRecord,
    chunks: &[ChunkRecord],
    tags: &[String],
    now: &str,
) -> Vec<DraftMemory> {
    let mut drafts = Vec::new();

    for chunk in chunks {
        let segments = split_chunk_into_segments(&chunk.content);
        for segment in segments.into_iter().take(3) {
            let normalized = normalize::normalize_content(&segment);
            if normalized.split_whitespace().count() < 5 {
                continue;
            }

            let memory_type = infer_memory_type(&normalized, tags);
            let title = infer_title(document, &normalized, &memory_type);
            let summary = summarize(&normalized);
            drafts.push(DraftMemory {
                memory_type: memory_type.clone(),
                scope: infer_scope(document),
                title,
                content: truncate_words(&normalized, 90),
                summary,
                document_id: document.id.clone(),
                source_path: document.source_path.clone(),
                source_refs: vec![chunk.id.clone()],
                tags: tags.to_vec(),
                entity_refs: extract_entities(&normalized, &document.source_path),
                confidence: infer_confidence(&normalized, &memory_type),
                importance: infer_importance(&normalized, tags, &memory_type),
                status: "active".to_string(),
                created_at: now.to_string(),
                updated_at: now.to_string(),
                occurred_at: infer_occurred_at(&normalized, &memory_type, now),
                normalized_key: normalize::normalize_content(&normalized),
            });
        }
    }

    if drafts.is_empty() {
        let summary = summarize(&document.title);
        drafts.push(DraftMemory {
            memory_type: "semantic".to_string(),
            scope: infer_scope(document),
            title: if document.title.is_empty() {
                "memory".to_string()
            } else {
                document.title.clone()
            },
            content: summary.clone(),
            summary,
            document_id: document.id.clone(),
            source_path: document.source_path.clone(),
            source_refs: Vec::new(),
            tags: tags.to_vec(),
            entity_refs: extract_entities(&document.title, &document.source_path),
            confidence: 0.6,
            importance: 0.5,
            status: "active".to_string(),
            created_at: now.to_string(),
            updated_at: now.to_string(),
            occurred_at: String::new(),
            normalized_key: normalize::normalize_content(&document.title),
        });
    }

    let mut deduped = Vec::new();
    let mut seen = HashSet::new();
    for draft in drafts {
        if seen.insert(draft.normalized_key.clone()) {
            deduped.push(draft);
        }
    }
    deduped
}

fn build_relations(memories: &[MemoryRecord], now: &str) -> Vec<MemoryRelationRecord> {
    memories
        .windows(2)
        .map(|pair| MemoryRelationRecord {
            id: Uuid::new_v4().to_string(),
            from_memory_id: pair[0].id.clone(),
            to_memory_id: pair[1].id.clone(),
            relation_type: "sequence".to_string(),
            weight: 0.5,
            created_at: now.to_string(),
        })
        .collect()
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

fn split_chunk_into_segments(value: &str) -> Vec<String> {
    let paragraphs = value
        .split("\n\n")
        .flat_map(|paragraph| paragraph.lines())
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    let mut segments = Vec::new();

    for paragraph in paragraphs {
        let words = paragraph.split_whitespace().count();
        if (8..=80).contains(&words) {
            segments.push(paragraph.to_string());
            continue;
        }

        let mut current = Vec::new();
        let mut current_words = 0usize;
        for sentence in paragraph.split_terminator(['.', '!', '?']) {
            let sentence = sentence.trim();
            if sentence.is_empty() {
                continue;
            }
            let sentence_words = sentence.split_whitespace().count();
            if current_words + sentence_words > 60 && !current.is_empty() {
                segments.push(current.join(". ") + ".");
                current.clear();
                current_words = 0;
            }
            current.push(sentence.to_string());
            current_words += sentence_words;
        }
        if !current.is_empty() {
            segments.push(current.join(". ") + ".");
        }
    }

    if segments.is_empty() {
        segments.push(truncate_words(value, 80));
    }
    segments
}

fn infer_memory_type(value: &str, tags: &[String]) -> String {
    let lower = value.to_lowercase();
    if tags.iter().any(|tag| tag.eq_ignore_ascii_case("working"))
        || ["todo", "next", "follow up", "wip"]
            .iter()
            .any(|term| lower.contains(term))
    {
        return "working".to_string();
    }
    if ["how", "run", "use", "configure", "step", "must", "should"]
        .iter()
        .any(|term| lower.contains(term))
    {
        return "procedural".to_string();
    }
    if [
        "released",
        "fixed",
        "incident",
        "happened",
        "today",
        "yesterday",
        "issue",
    ]
    .iter()
    .any(|term| lower.contains(term))
    {
        return "episodic".to_string();
    }
    "semantic".to_string()
}

fn infer_title(document: &DocumentRecord, value: &str, memory_type: &str) -> String {
    if !document.title.is_empty() {
        return document.title.clone();
    }

    let snippet = truncate_words(value, 10);
    match memory_type {
        "procedural" => format!("Procedure: {snippet}"),
        "episodic" => format!("Event: {snippet}"),
        "working" => format!("Working: {snippet}"),
        _ => snippet,
    }
}

fn infer_scope(document: &DocumentRecord) -> String {
    if document.source_path.is_empty() {
        "global".to_string()
    } else {
        "repo".to_string()
    }
}

fn summarize(value: &str) -> String {
    truncate_words(value, 18)
}

fn infer_confidence(value: &str, memory_type: &str) -> f32 {
    let word_count = value.split_whitespace().count();
    let length_bonus = if word_count >= 12 { 0.1 } else { 0.0 };
    let type_bonus = match memory_type {
        "procedural" => 0.15,
        "episodic" => 0.1,
        "working" => 0.05,
        _ => 0.08,
    };
    (0.55_f32 + length_bonus + type_bonus).min(0.95_f32)
}

fn infer_importance(value: &str, tags: &[String], memory_type: &str) -> f32 {
    let lower = value.to_lowercase();
    let tag_bonus = if tags.is_empty() { 0.0 } else { 0.1 };
    let keyword_bonus = ["must", "required", "critical", "important", "never"]
        .iter()
        .filter(|term| lower.contains(**term))
        .count() as f32
        * 0.05;
    let type_bonus = match memory_type {
        "procedural" => 0.1,
        "episodic" => 0.08,
        _ => 0.05,
    };
    (0.45 + tag_bonus + keyword_bonus + type_bonus).min(1.0)
}

fn infer_occurred_at(value: &str, memory_type: &str, now: &str) -> String {
    if memory_type == "episodic"
        || ["today", "yesterday", "released", "fixed"]
            .iter()
            .any(|term| value.to_lowercase().contains(term))
    {
        now.to_string()
    } else {
        String::new()
    }
}

fn extract_entities(value: &str, source_path: &str) -> Vec<String> {
    let mut entities = value
        .split_whitespace()
        .map(|word| {
            word.trim_matches(|char: char| {
                !char.is_alphanumeric() && char != '/' && char != '_' && char != '.'
            })
        })
        .filter(|word| {
            !word.is_empty()
                && (word.contains('/')
                    || word.contains('_')
                    || word.contains('.')
                    || word.chars().any(|char| char.is_uppercase()))
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();

    if !source_path.is_empty() {
        entities.push(source_path.to_string());
    }

    entities.sort();
    entities.dedup();
    entities
}

fn truncate_words(value: &str, max_words: usize) -> String {
    let words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() <= max_words {
        value.trim().to_string()
    } else {
        format!("{}...", words[..max_words].join(" "))
    }
}

fn sha256(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
