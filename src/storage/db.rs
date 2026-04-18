use std::sync::Arc;

use anyhow::Result;
use arrow_array::{ArrayRef, FixedSizeListArray, RecordBatch, StringArray};
use arrow_array::{Float32Array, UInt32Array};
use arrow_schema::{DataType, Field, Schema};
use chrono::Utc;
use futures::TryStreamExt;
use lancedb::query::{ExecutableQuery, QueryBase};
use lancedb::{connect, Connection, DistanceType, Table};
use serde::Serialize;

use crate::embeddings::provider::EmbeddingProvider;
use crate::paths::CtxPaths;
use crate::ranking::SearchCandidate;

#[derive(Clone)]
pub struct CtxDatabase {
    pub connection: Connection,
    pub documents: Table,
    pub chunks: Table,
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

pub async fn init_database(paths: &CtxPaths, provider: &impl EmbeddingProvider) -> Result<CtxDatabase> {
    paths.ensure().await?;
    let connection = connect(paths.db_dir.to_string_lossy().as_ref()).execute().await?;
    let dimension = provider.dimension().await? as i32;

    let documents = match connection.open_table("documents").execute().await {
        Ok(table) => table,
        Err(_) => connection
            .create_table("documents", vec![empty_documents_batch()?])
            .execute()
            .await?,
    };

    let chunks = match connection.open_table("chunks").execute().await {
        Ok(table) => table,
        Err(_) => connection
            .create_table("chunks", vec![empty_chunks_batch(dimension)?])
            .execute()
            .await?,
    };

    Ok(CtxDatabase {
        connection,
        documents,
        chunks,
    })
}

pub async fn insert_document(db: &CtxDatabase, document: &DocumentRecord) -> Result<()> {
    db.documents
        .add(vec![document_batch(std::slice::from_ref(document))?])
        .execute()
        .await?;
    Ok(())
}

pub async fn insert_chunks(db: &CtxDatabase, chunks: &[ChunkRecord], vector_dimension: i32) -> Result<()> {
    if chunks.is_empty() {
        return Ok(());
    }
    db.chunks
        .add(vec![chunks_batch(chunks, vector_dimension)?])
        .execute()
        .await?;
    Ok(())
}

pub async fn vector_search(db: &CtxDatabase, query: &[f32], limit: usize) -> Result<Vec<SearchCandidate>> {
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
        .flat_map(record_batch_to_candidates)
        .collect())
}

pub async fn list_chunks(db: &CtxDatabase) -> Result<Vec<SearchCandidate>> {
    let batches: Vec<RecordBatch> = db.chunks.query().execute().await?.try_collect().await?;
    Ok(batches
        .into_iter()
        .flat_map(record_batch_to_candidates)
        .collect())
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
    let schema = chunks_schema(dimension);
    Ok(RecordBatch::new_empty(schema))
}

fn document_batch(documents: &[DocumentRecord]) -> Result<RecordBatch> {
    let schema = empty_documents_batch()?.schema();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(documents.iter().map(|item| item.id.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.source_type.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.source_path.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.source_hash.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.title.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.tags.join(",")).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.created_at.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(documents.iter().map(|item| item.updated_at.clone()).collect::<Vec<_>>())),
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
            DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), dimension),
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
        Arc::new(StringArray::from(chunks.iter().map(|item| item.id.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(chunks.iter().map(|item| item.document_id.clone()).collect::<Vec<_>>())),
        Arc::new(UInt32Array::from(chunks.iter().map(|item| item.chunk_index).collect::<Vec<_>>())),
        Arc::new(StringArray::from(chunks.iter().map(|item| item.content.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(chunks.iter().map(|item| item.title.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(chunks.iter().map(|item| item.source_path.clone()).collect::<Vec<_>>())),
        Arc::new(StringArray::from(chunks.iter().map(|item| item.tags.join(",")).collect::<Vec<_>>())),
        Arc::new(StringArray::from(
            chunks
                .iter()
                .map(|item| {
                    if item.created_at.is_empty() {
                        Utc::now().to_rfc3339()
                    } else {
                        item.created_at.clone()
                    }
                })
                .collect::<Vec<_>>(),
        )),
        Arc::new(vectors),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn record_batch_to_candidates(batch: RecordBatch) -> Vec<SearchCandidate> {
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
    let distance = batch
        .column_by_name("_distance")
        .and_then(|column| column.as_any().downcast_ref::<Float32Array>());

    let Some(ids) = ids else { return Vec::new() };
    let Some(doc_ids) = doc_ids else { return Vec::new() };
    let Some(content) = content else { return Vec::new() };
    let Some(title) = title else { return Vec::new() };
    let Some(source_path) = source_path else { return Vec::new() };
    let Some(tags) = tags else { return Vec::new() };

    (0..batch.num_rows())
        .map(|index| SearchCandidate {
            id: ids.value(index).to_string(),
            document_id: doc_ids.value(index).to_string(),
            title: Some(title.value(index).to_string()).filter(|value| !value.is_empty()),
            source_path: Some(source_path.value(index).to_string()).filter(|value| !value.is_empty()),
            tags: tags
                .value(index)
                .split(',')
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect(),
            content: content.value(index).to_string(),
            vector_score: distance.map(|array| 1.0 - array.value(index)).unwrap_or(0.0),
            keyword_score: 0.0,
            final_score: 0.0,
        })
        .collect()
}
