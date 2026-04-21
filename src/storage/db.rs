use std::sync::Arc;

use anyhow::Result;
use arrow_array::UInt32Array;
use arrow_array::{ArrayRef, FixedSizeListArray, Float32Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType, Table};
use serde::Serialize;

use crate::embeddings::provider::EmbeddingProvider;
use crate::paths::CtxPaths;
use crate::ranking::SearchCandidate;

const SCHEMA_VERSION: &str = "2";

#[derive(Clone)]
pub struct CtxDatabase {
    pub connection: Connection,
    pub metadata: Table,
    pub documents: Table,
    pub chunks: Table,
    pub memories: Table,
    pub memory_relations: Table,
    pub memory_access_log: Table,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetadataRecord {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentRecord {
    pub id: String,
    pub source_type: String,
    pub source_path: String,
    pub source_hash: String,
    pub title: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkRecord {
    pub id: String,
    pub document_id: String,
    pub chunk_index: u32,
    pub content: String,
    pub title: String,
    pub source_path: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryRecord {
    pub id: String,
    pub memory_type: String,
    pub scope: String,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub document_id: String,
    pub source_path: String,
    pub source_refs: Vec<String>,
    pub tags: Vec<String>,
    pub entity_refs: Vec<String>,
    pub confidence: f32,
    pub importance: f32,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub occurred_at: String,
    pub last_accessed_at: String,
    pub access_count: u32,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryRelationRecord {
    pub id: String,
    pub from_memory_id: String,
    pub to_memory_id: String,
    pub relation_type: String,
    pub weight: f32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryAccessLogRecord {
    pub id: String,
    pub memory_id: String,
    pub query: String,
    pub retrieval_mode: String,
    pub rank: u32,
    pub score: f32,
    pub accessed_at: String,
}

pub async fn init_database(
    paths: &CtxPaths,
    provider: &(impl EmbeddingProvider + ?Sized),
) -> Result<CtxDatabase> {
    paths.ensure().await?;
    let connection = connect(paths.db_dir.to_string_lossy().as_ref())
        .execute()
        .await?;
    let dimension = provider.dimension().await? as i32;

    let metadata = open_or_create_table(&connection, "metadata", empty_metadata_batch()?).await?;
    let documents =
        open_or_create_table(&connection, "documents", empty_documents_batch()?).await?;
    let chunks =
        open_or_create_table(&connection, "chunks", empty_chunks_batch(dimension)?).await?;
    let memories =
        open_or_create_table(&connection, "memories", empty_memories_batch(dimension)?).await?;
    let memory_relations = open_or_create_table(
        &connection,
        "memory_relations",
        empty_memory_relations_batch()?,
    )
    .await?;
    let memory_access_log = open_or_create_table(
        &connection,
        "memory_access_log",
        empty_memory_access_log_batch()?,
    )
    .await?;

    let db = CtxDatabase {
        connection,
        metadata,
        documents,
        chunks,
        memories,
        memory_relations,
        memory_access_log,
    };

    ensure_metadata(&db, "schema_version", SCHEMA_VERSION).await?;
    ensure_metadata(&db, "app_version", env!("CARGO_PKG_VERSION")).await?;

    Ok(db)
}

pub async fn insert_document(db: &CtxDatabase, document: &DocumentRecord) -> Result<()> {
    db.documents
        .add(vec![document_batch(std::slice::from_ref(document))?])
        .execute()
        .await?;
    Ok(())
}

pub async fn insert_chunks(
    db: &CtxDatabase,
    chunks: &[ChunkRecord],
    vector_dimension: i32,
) -> Result<()> {
    if chunks.is_empty() {
        return Ok(());
    }
    db.chunks
        .add(vec![chunks_batch(chunks, vector_dimension)?])
        .execute()
        .await?;
    Ok(())
}

pub async fn insert_memories(
    db: &CtxDatabase,
    memories: &[MemoryRecord],
    vector_dimension: i32,
) -> Result<()> {
    if memories.is_empty() {
        return Ok(());
    }
    db.memories
        .add(vec![memories_batch(memories, vector_dimension)?])
        .execute()
        .await?;
    Ok(())
}

pub async fn insert_memory_relations(
    db: &CtxDatabase,
    relations: &[MemoryRelationRecord],
) -> Result<()> {
    if relations.is_empty() {
        return Ok(());
    }
    db.memory_relations
        .add(vec![memory_relations_batch(relations)?])
        .execute()
        .await?;
    Ok(())
}

pub async fn record_memory_access(
    db: &CtxDatabase,
    records: &[MemoryAccessLogRecord],
) -> Result<()> {
    if records.is_empty() {
        return Ok(());
    }

    db.memory_access_log
        .add(vec![memory_access_log_batch(records)?])
        .execute()
        .await?;

    for record in records {
        db.memories
            .update()
            .only_if(format!("id = '{}'", sql_string(&record.memory_id)))
            .column(
                "last_accessed_at",
                format!("'{}'", sql_string(&record.accessed_at)),
            )
            .column("access_count", "access_count + 1")
            .execute()
            .await?;
    }

    Ok(())
}

pub async fn vector_search(
    db: &CtxDatabase,
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchCandidate>> {
    let batches: Vec<RecordBatch> = db
        .chunks
        .query()
        .nearest_to(query)?
        .distance_type(DistanceType::Cosine)
        .limit(limit)
        .execute()
        .await?
        .try_collect()
        .await?;

    Ok(batches
        .into_iter()
        .flat_map(record_batch_to_chunk_candidates)
        .collect())
}

pub async fn vector_search_memories(
    db: &CtxDatabase,
    query: &[f32],
    limit: usize,
) -> Result<Vec<SearchCandidate>> {
    let batches: Vec<RecordBatch> = db
        .memories
        .query()
        .nearest_to(query)?
        .distance_type(DistanceType::Cosine)
        .limit(limit)
        .execute()
        .await?
        .try_collect()
        .await?;

    Ok(batches
        .into_iter()
        .flat_map(record_batch_to_memory_candidates)
        .collect())
}

pub async fn list_chunks(db: &CtxDatabase) -> Result<Vec<SearchCandidate>> {
    let batches: Vec<RecordBatch> = db.chunks.query().execute().await?.try_collect().await?;
    Ok(batches
        .into_iter()
        .flat_map(record_batch_to_chunk_candidates)
        .collect())
}

pub async fn list_memories(db: &CtxDatabase) -> Result<Vec<SearchCandidate>> {
    let batches: Vec<RecordBatch> = db.memories.query().execute().await?.try_collect().await?;
    Ok(batches
        .into_iter()
        .flat_map(record_batch_to_memory_candidates)
        .collect())
}

pub async fn list_metadata(db: &CtxDatabase) -> Result<Vec<MetadataRecord>> {
    let batches: Vec<RecordBatch> = db.metadata.query().execute().await?.try_collect().await?;
    Ok(batches
        .into_iter()
        .flat_map(record_batch_to_metadata)
        .collect())
}

async fn ensure_metadata(db: &CtxDatabase, key: &str, value: &str) -> Result<()> {
    db.metadata
        .delete(&format!("key = '{}'", sql_string(key)))
        .await?;
    let record = MetadataRecord {
        key: key.to_string(),
        value: value.to_string(),
        updated_at: Utc::now().to_rfc3339(),
    };
    db.metadata
        .add(vec![metadata_batch(std::slice::from_ref(&record))?])
        .execute()
        .await?;
    Ok(())
}

async fn open_or_create_table(
    connection: &Connection,
    name: &str,
    batch: RecordBatch,
) -> Result<Table> {
    match connection.open_table(name).execute().await {
        Ok(table) => Ok(table),
        Err(_) => Ok(connection.create_table(name, vec![batch]).execute().await?),
    }
}

fn empty_metadata_batch() -> Result<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("key", DataType::Utf8, false),
        Field::new("value", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
    ]));
    Ok(RecordBatch::new_empty(schema))
}

fn empty_documents_batch() -> Result<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("source_type", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("source_hash", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("tags", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
    ]));
    Ok(RecordBatch::new_empty(schema))
}

fn empty_chunks_batch(dimension: i32) -> Result<RecordBatch> {
    Ok(RecordBatch::new_empty(chunks_schema(dimension)))
}

fn empty_memories_batch(dimension: i32) -> Result<RecordBatch> {
    Ok(RecordBatch::new_empty(memories_schema(dimension)))
}

fn empty_memory_relations_batch() -> Result<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("from_memory_id", DataType::Utf8, false),
        Field::new("to_memory_id", DataType::Utf8, false),
        Field::new("relation_type", DataType::Utf8, false),
        Field::new("weight", DataType::Float32, false),
        Field::new("created_at", DataType::Utf8, false),
    ]));
    Ok(RecordBatch::new_empty(schema))
}

fn empty_memory_access_log_batch() -> Result<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("memory_id", DataType::Utf8, false),
        Field::new("query", DataType::Utf8, false),
        Field::new("retrieval_mode", DataType::Utf8, false),
        Field::new("rank", DataType::UInt32, false),
        Field::new("score", DataType::Float32, false),
        Field::new("accessed_at", DataType::Utf8, false),
    ]));
    Ok(RecordBatch::new_empty(schema))
}

fn metadata_batch(records: &[MetadataRecord]) -> Result<RecordBatch> {
    let schema = empty_metadata_batch()?.schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.key.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.value.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.updated_at.clone())
                .collect::<Vec<_>>(),
        )),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn document_batch(documents: &[DocumentRecord]) -> Result<RecordBatch> {
    let schema = empty_documents_batch()?.schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.source_type.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.source_path.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.source_hash.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.tags.join(","))
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.created_at.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            documents
                .iter()
                .map(|item| item.updated_at.clone())
                .collect::<Vec<_>>(),
        )),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn chunks_schema(dimension: i32) -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("document_id", DataType::Utf8, false),
        Field::new("chunk_index", DataType::UInt32, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("tags", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dimension,
            ),
            true,
        ),
    ]))
}

fn chunks_batch(chunks: &[ChunkRecord], dimension: i32) -> Result<RecordBatch> {
    let schema = chunks_schema(dimension);
    let values = Float32Array::from(
        chunks
            .iter()
            .flat_map(|item| item.vector.clone())
            .collect::<Vec<_>>(),
    );
    let vectors = FixedSizeListArray::try_new(
        Arc::new(Field::new("item", DataType::Float32, true)),
        dimension,
        Arc::new(values),
        None,
    )?;
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| item.document_id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt32Array::from(
            chunks
                .iter()
                .map(|item| item.chunk_index)
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| item.content.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| item.source_path.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| item.tags.join(","))
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| default_timestamp(&item.created_at))
                .collect::<Vec<_>>(),
        )),
        Arc::new(vectors),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn memories_schema(dimension: i32) -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("memory_type", DataType::Utf8, false),
        Field::new("scope", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("content", DataType::Utf8, false),
        Field::new("summary", DataType::Utf8, false),
        Field::new("document_id", DataType::Utf8, false),
        Field::new("source_path", DataType::Utf8, false),
        Field::new("source_refs", DataType::Utf8, false),
        Field::new("tags", DataType::Utf8, false),
        Field::new("entity_refs", DataType::Utf8, false),
        Field::new("confidence", DataType::Float32, false),
        Field::new("importance", DataType::Float32, false),
        Field::new("status", DataType::Utf8, false),
        Field::new("created_at", DataType::Utf8, false),
        Field::new("updated_at", DataType::Utf8, false),
        Field::new("occurred_at", DataType::Utf8, false),
        Field::new("last_accessed_at", DataType::Utf8, false),
        Field::new("access_count", DataType::UInt32, false),
        Field::new(
            "vector",
            DataType::FixedSizeList(
                Arc::new(Field::new("item", DataType::Float32, true)),
                dimension,
            ),
            true,
        ),
    ]))
}

fn memories_batch(memories: &[MemoryRecord], dimension: i32) -> Result<RecordBatch> {
    let schema = memories_schema(dimension);
    let values = Float32Array::from(
        memories
            .iter()
            .flat_map(|item| item.vector.clone())
            .collect::<Vec<_>>(),
    );
    let vectors = FixedSizeListArray::try_new(
        Arc::new(Field::new("item", DataType::Float32, true)),
        dimension,
        Arc::new(values),
        None,
    )?;
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.memory_type.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.scope.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.title.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.content.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.summary.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.document_id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.source_path.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.source_refs.join(","))
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.tags.join(","))
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.entity_refs.join(","))
                .collect::<Vec<_>>(),
        )),
        Arc::new(Float32Array::from(
            memories
                .iter()
                .map(|item| item.confidence)
                .collect::<Vec<_>>(),
        )),
        Arc::new(Float32Array::from(
            memories
                .iter()
                .map(|item| item.importance)
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.status.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| default_timestamp(&item.created_at))
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| default_timestamp(&item.updated_at))
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.occurred_at.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            memories
                .iter()
                .map(|item| item.last_accessed_at.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt32Array::from(
            memories
                .iter()
                .map(|item| item.access_count)
                .collect::<Vec<_>>(),
        )),
        Arc::new(vectors),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn memory_relations_batch(relations: &[MemoryRelationRecord]) -> Result<RecordBatch> {
    let schema = empty_memory_relations_batch()?.schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(
            relations
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            relations
                .iter()
                .map(|item| item.from_memory_id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            relations
                .iter()
                .map(|item| item.to_memory_id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            relations
                .iter()
                .map(|item| item.relation_type.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(Float32Array::from(
            relations.iter().map(|item| item.weight).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            relations
                .iter()
                .map(|item| default_timestamp(&item.created_at))
                .collect::<Vec<_>>(),
        )),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn memory_access_log_batch(records: &[MemoryAccessLogRecord]) -> Result<RecordBatch> {
    let schema = empty_memory_access_log_batch()?.schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.memory_id.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.query.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| item.retrieval_mode.clone())
                .collect::<Vec<_>>(),
        )),
        Arc::new(UInt32Array::from(
            records.iter().map(|item| item.rank).collect::<Vec<_>>(),
        )),
        Arc::new(Float32Array::from(
            records.iter().map(|item| item.score).collect::<Vec<_>>(),
        )),
        Arc::new(StringArray::from(
            records
                .iter()
                .map(|item| default_timestamp(&item.accessed_at))
                .collect::<Vec<_>>(),
        )),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn record_batch_to_metadata(batch: RecordBatch) -> Vec<MetadataRecord> {
    let keys = batch
        .column_by_name("key")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let values = batch
        .column_by_name("value")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let updated_at = batch
        .column_by_name("updated_at")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());

    let Some(keys) = keys else { return Vec::new() };
    let Some(values) = values else {
        return Vec::new();
    };
    let Some(updated_at) = updated_at else {
        return Vec::new();
    };

    (0..batch.num_rows())
        .map(|index| MetadataRecord {
            key: keys.value(index).to_string(),
            value: values.value(index).to_string(),
            updated_at: updated_at.value(index).to_string(),
        })
        .collect()
}

fn record_batch_to_chunk_candidates(batch: RecordBatch) -> Vec<SearchCandidate> {
    let ids = batch
        .column_by_name("id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let doc_ids = batch
        .column_by_name("document_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let content = batch
        .column_by_name("content")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let title = batch
        .column_by_name("title")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let source_path = batch
        .column_by_name("source_path")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let tags = batch
        .column_by_name("tags")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let created_at = batch
        .column_by_name("created_at")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let distance = batch
        .column_by_name("_distance")
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>());

    let Some(ids) = ids else { return Vec::new() };
    let Some(doc_ids) = doc_ids else {
        return Vec::new();
    };
    let Some(content) = content else {
        return Vec::new();
    };
    let Some(title) = title else {
        return Vec::new();
    };
    let Some(source_path) = source_path else {
        return Vec::new();
    };
    let Some(tags) = tags else { return Vec::new() };

    (0..batch.num_rows())
        .map(|index| SearchCandidate {
            id: ids.value(index).to_string(),
            kind: "chunk".to_string(),
            document_id: doc_ids.value(index).to_string(),
            title: non_empty(title.value(index)),
            summary: non_empty(&truncate_words(content.value(index), 24)),
            memory_type: None,
            scope: None,
            status: None,
            source_path: non_empty(source_path.value(index)),
            source_refs: vec![ids.value(index).to_string()],
            tags: split_csv(tags.value(index)),
            entity_refs: Vec::new(),
            content: content.value(index).to_string(),
            confidence: 0.0,
            importance: 0.0,
            access_count: 0,
            created_at: created_at.and_then(|value| non_empty(value.value(index))),
            updated_at: None,
            occurred_at: None,
            last_accessed_at: None,
            vector_score: distance
                .map(|array| 1.0 - array.value(index))
                .unwrap_or(0.0),
            keyword_score: 0.0,
            final_score: 0.0,
        })
        .collect()
}

fn record_batch_to_memory_candidates(batch: RecordBatch) -> Vec<SearchCandidate> {
    let ids = batch
        .column_by_name("id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let memory_type = batch
        .column_by_name("memory_type")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let scope = batch
        .column_by_name("scope")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let title = batch
        .column_by_name("title")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let content = batch
        .column_by_name("content")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let summary = batch
        .column_by_name("summary")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let doc_ids = batch
        .column_by_name("document_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let source_path = batch
        .column_by_name("source_path")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let source_refs = batch
        .column_by_name("source_refs")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let tags = batch
        .column_by_name("tags")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let entity_refs = batch
        .column_by_name("entity_refs")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let confidence = batch
        .column_by_name("confidence")
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>());
    let importance = batch
        .column_by_name("importance")
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>());
    let status = batch
        .column_by_name("status")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let created_at = batch
        .column_by_name("created_at")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let updated_at = batch
        .column_by_name("updated_at")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let occurred_at = batch
        .column_by_name("occurred_at")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let last_accessed_at = batch
        .column_by_name("last_accessed_at")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>());
    let access_count = batch
        .column_by_name("access_count")
        .and_then(|column| column.as_any().downcast_ref::<UInt32Array>());
    let distance = batch
        .column_by_name("_distance")
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>());

    let Some(ids) = ids else { return Vec::new() };
    let Some(memory_type) = memory_type else {
        return Vec::new();
    };
    let Some(scope) = scope else {
        return Vec::new();
    };
    let Some(title) = title else {
        return Vec::new();
    };
    let Some(content) = content else {
        return Vec::new();
    };
    let Some(summary) = summary else {
        return Vec::new();
    };
    let Some(doc_ids) = doc_ids else {
        return Vec::new();
    };
    let Some(source_path) = source_path else {
        return Vec::new();
    };
    let Some(source_refs) = source_refs else {
        return Vec::new();
    };
    let Some(tags) = tags else { return Vec::new() };
    let Some(entity_refs) = entity_refs else {
        return Vec::new();
    };
    let Some(confidence) = confidence else {
        return Vec::new();
    };
    let Some(importance) = importance else {
        return Vec::new();
    };
    let Some(status) = status else {
        return Vec::new();
    };
    let Some(access_count) = access_count else {
        return Vec::new();
    };

    (0..batch.num_rows())
        .map(|index| SearchCandidate {
            id: ids.value(index).to_string(),
            kind: "memory".to_string(),
            document_id: doc_ids.value(index).to_string(),
            title: non_empty(title.value(index)),
            summary: non_empty(summary.value(index)),
            memory_type: non_empty(memory_type.value(index)),
            scope: non_empty(scope.value(index)),
            status: non_empty(status.value(index)),
            source_path: non_empty(source_path.value(index)),
            source_refs: split_csv(source_refs.value(index)),
            tags: split_csv(tags.value(index)),
            entity_refs: split_csv(entity_refs.value(index)),
            content: content.value(index).to_string(),
            confidence: confidence.value(index),
            importance: importance.value(index),
            access_count: access_count.value(index),
            created_at: created_at.and_then(|value| non_empty(value.value(index))),
            updated_at: updated_at.and_then(|value| non_empty(value.value(index))),
            occurred_at: occurred_at.and_then(|value| non_empty(value.value(index))),
            last_accessed_at: last_accessed_at.and_then(|value| non_empty(value.value(index))),
            vector_score: distance
                .map(|array| 1.0 - array.value(index))
                .unwrap_or(0.0),
            keyword_score: 0.0,
            final_score: 0.0,
        })
        .collect()
}

fn default_timestamp(value: &str) -> String {
    if value.is_empty() {
        Utc::now().to_rfc3339()
    } else {
        value.to_string()
    }
}

fn split_csv(value: &str) -> Vec<String> {
    value
        .split(',')
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn truncate_words(value: &str, max_words: usize) -> String {
    let words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() <= max_words {
        value.trim().to_string()
    } else {
        format!("{}...", words[..max_words].join(" "))
    }
}

fn sql_string(value: &str) -> String {
    value.replace('\'', "''")
}

pub fn recency_score(value: Option<&str>) -> f32 {
    let Some(value) = value else { return 0.0 };
    let Ok(timestamp) = DateTime::parse_from_rfc3339(value) else {
        return 0.0;
    };
    let elapsed = Utc::now().signed_duration_since(timestamp.with_timezone(&Utc));
    let days = elapsed.num_days().max(0) as f32;
    1.0 / (1.0 + (days / 30.0))
}
