use std::collections::HashMap;

use anyhow::Result;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{
    config::CtxConfig,
    embeddings::provider::EmbeddingProvider,
    ranking::{rank_results, SearchCandidate},
    storage::{self, CtxDatabase},
};

#[derive(Debug, Serialize, JsonSchema)]
pub struct SearchResult {
    pub ok: bool,
    pub query: String,
    pub count: usize,
    pub results: Vec<SearchCandidate>,
}

pub async fn run_search(
    db: &CtxDatabase,
    provider: &(impl EmbeddingProvider + ?Sized),
    config: &CtxConfig,
    query: &str,
    top_k: usize,
    tags: &[String],
) -> Result<SearchResult> {
    let query_vector = provider.embed_query(query).await?;
    let vector_matches = storage::vector_search(db, &query_vector, top_k * 4).await?;
    let keyword_scores = keyword_scores(storage::list_chunks(db).await?, query, tags);
    let filtered_vector_matches = vector_matches
        .into_iter()
        .filter(|candidate| tags.is_empty() || tags.iter().all(|tag| candidate.tags.contains(tag)))
        .collect::<Vec<_>>();
    let results = rank_results(
        filtered_vector_matches,
        keyword_scores,
        top_k,
        config.ranking.vector_weight,
        config.ranking.keyword_weight,
    );

    Ok(SearchResult {
        ok: true,
        query: query.to_string(),
        count: results.len(),
        results,
    })
}

fn keyword_scores(
    chunks: Vec<SearchCandidate>,
    query: &str,
    tags: &[String],
) -> HashMap<String, f32> {
    let terms = query
        .split_whitespace()
        .map(|value| value.to_lowercase())
        .collect::<Vec<_>>();
    let mut scores = HashMap::new();

    for chunk in chunks {
        if !tags.is_empty() && !tags.iter().all(|tag| chunk.tags.contains(tag)) {
            continue;
        }
        let haystack = chunk.content.to_lowercase();
        let hits = terms
            .iter()
            .filter(|term| haystack.contains(term.as_str()))
            .count();
        if hits > 0 {
            scores.insert(chunk.id, hits as f32 / terms.len() as f32);
        }
    }

    scores
}
