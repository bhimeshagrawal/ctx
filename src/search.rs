use std::collections::{HashMap, HashSet};

use anyhow::Result;
use schemars::JsonSchema;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    config::CtxConfig,
    embeddings::provider::EmbeddingProvider,
    ranking::{rank_results, SearchCandidate},
    storage::{self, CtxDatabase, MemoryAccessLogRecord},
};

#[derive(Debug, Clone, Serialize, JsonSchema, Default)]
pub struct ContextPack {
    pub relevant_facts: Vec<SearchCandidate>,
    pub relevant_procedures: Vec<SearchCandidate>,
    pub relevant_recent_events: Vec<SearchCandidate>,
    pub evidence_snippets: Vec<SearchCandidate>,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchResult {
    pub ok: bool,
    pub mode: String,
    pub query: String,
    pub count: usize,
    pub results: Vec<SearchCandidate>,
    pub context_pack: ContextPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum QueryIntent {
    Exact,
    Procedure,
    Event,
    General,
}

pub async fn run_search(
    db: &CtxDatabase,
    provider: &(impl EmbeddingProvider + ?Sized),
    config: &CtxConfig,
    query: &str,
    top_k: usize,
    tags: &[String],
    raw: bool,
) -> Result<SearchResult> {
    if raw {
        return run_raw_chunk_search(db, provider, config, query, top_k, tags).await;
    }

    let intent = classify_query_intent(query);
    let query_vector = provider.embed_query(query).await?;
    let vector_matches = storage::vector_search_memories(db, &query_vector, top_k * 6).await?;
    let all_memories = storage::list_memories(db).await?;
    let lexical_ranked = lexical_memory_candidates(&all_memories, query, tags, intent, top_k * 6);
    let max_results = top_k.min(config.retrieval.max_memories.max(1));
    let results = rank_memory_candidates(
        merge_candidates(vector_matches, lexical_ranked),
        query,
        tags,
        intent,
        config,
        max_results,
    );

    let evidence_snippets = build_evidence_snippets(
        db,
        &results,
        config.retrieval.max_evidence_snippets,
        config.retrieval.max_evidence_words,
    )
    .await?;
    let context_pack = build_context_pack(config, results.clone(), evidence_snippets);
    let access_logs = results
        .iter()
        .enumerate()
        .map(|(index, candidate)| MemoryAccessLogRecord {
            id: Uuid::new_v4().to_string(),
            memory_id: candidate.id.clone(),
            query: query.to_string(),
            retrieval_mode: "memory".to_string(),
            rank: index as u32,
            score: candidate.final_score,
            accessed_at: chrono::Utc::now().to_rfc3339(),
        })
        .collect::<Vec<_>>();
    storage::record_memory_access(db, &access_logs).await?;

    Ok(SearchResult {
        ok: true,
        mode: config.retrieval.default_mode.clone(),
        query: query.to_string(),
        count: results.len(),
        results,
        context_pack,
    })
}

async fn run_raw_chunk_search(
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
        mode: "raw".to_string(),
        query: query.to_string(),
        count: results.len(),
        context_pack: ContextPack {
            evidence_snippets: results.clone(),
            ..ContextPack::default()
        },
        results,
    })
}

fn keyword_scores(
    chunks: Vec<SearchCandidate>,
    query: &str,
    tags: &[String],
) -> HashMap<String, f32> {
    let terms = query_terms(query);
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
            scores.insert(chunk.id, hits as f32 / terms.len().max(1) as f32);
        }
    }

    scores
}

fn lexical_memory_candidates(
    candidates: &[SearchCandidate],
    query: &str,
    tags: &[String],
    intent: QueryIntent,
    limit: usize,
) -> Vec<SearchCandidate> {
    let mut ranked = candidates
        .iter()
        .filter(|candidate| tags.is_empty() || tags.iter().all(|tag| candidate.tags.contains(tag)))
        .cloned()
        .map(|mut candidate| {
            candidate.keyword_score = lexical_score(&candidate, query, intent);
            candidate
        })
        .filter(|candidate| candidate.keyword_score > 0.0)
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| right.keyword_score.total_cmp(&left.keyword_score));
    ranked.truncate(limit);
    ranked
}

fn rank_memory_candidates(
    candidates: Vec<SearchCandidate>,
    query: &str,
    tags: &[String],
    intent: QueryIntent,
    config: &CtxConfig,
    top_k: usize,
) -> Vec<SearchCandidate> {
    let mut ranked = candidates
        .into_iter()
        .filter(|candidate| tags.is_empty() || tags.iter().all(|tag| candidate.tags.contains(tag)))
        .map(|mut candidate| {
            let lexical = lexical_score(&candidate, query, intent);
            let title = title_score(&candidate, query);
            let path = path_score(&candidate, query);
            let recency = storage::recency_score(
                candidate
                    .occurred_at
                    .as_deref()
                    .or(candidate.updated_at.as_deref())
                    .or(candidate.created_at.as_deref()),
            );
            let access = access_score(candidate.access_count);
            let scope = scope_score(&candidate, intent);
            candidate.keyword_score = lexical;
            candidate.final_score = (candidate.vector_score * config.ranking.vector_weight)
                + (lexical * config.ranking.keyword_weight)
                + (title * config.ranking.title_weight)
                + (path * config.ranking.path_weight)
                + (recency * config.ranking.recency_weight)
                + (candidate.importance * config.ranking.importance_weight)
                + (candidate.confidence * config.ranking.confidence_weight)
                + (access * config.ranking.access_weight)
                + (scope * config.ranking.scope_weight)
                + intent_bonus(&candidate, intent);
            candidate
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| right.final_score.total_cmp(&left.final_score));
    ranked.truncate(top_k);
    ranked
}

fn build_context_pack(
    config: &CtxConfig,
    results: Vec<SearchCandidate>,
    evidence_snippets: Vec<SearchCandidate>,
) -> ContextPack {
    let mut facts = Vec::new();
    let mut procedures = Vec::new();
    let mut events = Vec::new();
    let mut budget = config.retrieval.context_word_budget;

    for candidate in results {
        if budget == 0 {
            break;
        }

        let mut candidate = candidate;
        candidate.content = truncate_words(&candidate.content, config.retrieval.max_memory_words);
        let words = candidate.content.split_whitespace().count();
        if words > budget {
            candidate.content = truncate_words(&candidate.content, budget);
        }
        let consumed = candidate.content.split_whitespace().count();
        budget = budget.saturating_sub(consumed);

        match candidate.memory_type.as_deref() {
            Some("procedural") if procedures.len() < 2 => procedures.push(candidate),
            Some("episodic") if events.len() < 2 => events.push(candidate),
            _ if facts.len() < 2 => facts.push(candidate),
            Some("procedural") if facts.len() < 2 => facts.push(candidate),
            Some("episodic") if facts.len() < 2 => facts.push(candidate),
            _ => {}
        }
    }

    ContextPack {
        relevant_facts: facts,
        relevant_procedures: procedures,
        relevant_recent_events: events,
        evidence_snippets,
    }
}

async fn build_evidence_snippets(
    db: &CtxDatabase,
    results: &[SearchCandidate],
    max_snippets: usize,
    max_words: usize,
) -> Result<Vec<SearchCandidate>> {
    let needs_evidence = results
        .iter()
        .any(|candidate| candidate.confidence < 0.72 || candidate.final_score < 0.55);
    if !needs_evidence || max_snippets == 0 {
        return Ok(Vec::new());
    }

    let chunk_ids = results
        .iter()
        .flat_map(|candidate| candidate.source_refs.iter().cloned())
        .collect::<HashSet<_>>();
    if chunk_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut chunks = storage::list_chunks(db)
        .await?
        .into_iter()
        .filter(|chunk| chunk_ids.contains(&chunk.id))
        .collect::<Vec<_>>();
    chunks.truncate(max_snippets);
    for chunk in &mut chunks {
        chunk.content = truncate_words(&chunk.content, max_words);
        chunk.summary = Some(truncate_words(&chunk.content, 18));
    }
    Ok(chunks)
}

fn merge_candidates(
    vector_matches: Vec<SearchCandidate>,
    lexical_matches: Vec<SearchCandidate>,
) -> Vec<SearchCandidate> {
    let mut merged = HashMap::new();

    for candidate in lexical_matches {
        merged.insert(candidate.id.clone(), candidate);
    }

    for candidate in vector_matches {
        merged
            .entry(candidate.id.clone())
            .and_modify(|current: &mut SearchCandidate| {
                current.vector_score = current.vector_score.max(candidate.vector_score);
                if current.title.is_none() {
                    current.title = candidate.title.clone();
                }
                if current.summary.is_none() {
                    current.summary = candidate.summary.clone();
                }
            })
            .or_insert(candidate);
    }

    merged.into_values().collect()
}

fn classify_query_intent(query: &str) -> QueryIntent {
    let lower = query.to_lowercase();
    if lower.contains('/') || lower.contains("::") || lower.contains(".rs") || lower.contains('#') {
        return QueryIntent::Exact;
    }
    if [
        "how",
        "steps",
        "run",
        "configure",
        "setup",
        "use",
        "process",
    ]
    .iter()
    .any(|term| lower.contains(term))
    {
        return QueryIntent::Procedure;
    }
    if [
        "what happened",
        "recent",
        "when",
        "released",
        "fixed",
        "incident",
        "issue",
    ]
    .iter()
    .any(|term| lower.contains(term))
    {
        return QueryIntent::Event;
    }
    QueryIntent::General
}

fn lexical_score(candidate: &SearchCandidate, query: &str, intent: QueryIntent) -> f32 {
    let terms = query_terms(query);
    if terms.is_empty() {
        return 0.0;
    }

    let content = candidate.content.to_lowercase();
    let summary = candidate.summary.clone().unwrap_or_default().to_lowercase();
    let entity_refs = candidate
        .entity_refs
        .iter()
        .map(|item| item.to_lowercase())
        .collect::<Vec<_>>();
    let hits = terms
        .iter()
        .filter(|term| {
            content.contains(term.as_str())
                || summary.contains(term.as_str())
                || entity_refs
                    .iter()
                    .any(|entity| entity.contains(term.as_str()))
        })
        .count();

    let base = hits as f32 / terms.len() as f32;
    match intent {
        QueryIntent::Exact if path_score(candidate, query) > 0.0 => (base + 0.25).min(1.0),
        QueryIntent::Procedure if candidate.memory_type.as_deref() == Some("procedural") => {
            (base + 0.15).min(1.0)
        }
        QueryIntent::Event if candidate.memory_type.as_deref() == Some("episodic") => {
            (base + 0.15).min(1.0)
        }
        _ => base.min(1.0),
    }
}

fn title_score(candidate: &SearchCandidate, query: &str) -> f32 {
    let Some(title) = candidate.title.as_ref() else {
        return 0.0;
    };
    text_match_score(title, query)
}

fn path_score(candidate: &SearchCandidate, query: &str) -> f32 {
    let Some(path) = candidate.source_path.as_ref() else {
        return 0.0;
    };
    text_match_score(path, query)
}

fn scope_score(candidate: &SearchCandidate, intent: QueryIntent) -> f32 {
    match (candidate.scope.as_deref(), intent) {
        (Some("repo"), QueryIntent::Exact) => 1.0,
        (Some("repo"), QueryIntent::Procedure) => 0.8,
        (Some("global"), QueryIntent::General) => 0.7,
        (Some("workspace"), _) => 0.6,
        _ => 0.4,
    }
}

fn access_score(access_count: u32) -> f32 {
    ((access_count + 1) as f32).ln_1p() / 2.0
}

fn intent_bonus(candidate: &SearchCandidate, intent: QueryIntent) -> f32 {
    match intent {
        QueryIntent::Procedure if candidate.memory_type.as_deref() == Some("procedural") => 0.1,
        QueryIntent::Event if candidate.memory_type.as_deref() == Some("episodic") => 0.1,
        QueryIntent::Exact if candidate.scope.as_deref() == Some("repo") => 0.05,
        _ => 0.0,
    }
}

fn text_match_score(value: &str, query: &str) -> f32 {
    let haystack = value.to_lowercase();
    let terms = query_terms(query);
    if terms.is_empty() {
        return 0.0;
    }
    let hits = terms
        .iter()
        .filter(|term| haystack.contains(term.as_str()))
        .count();
    hits as f32 / terms.len() as f32
}

fn query_terms(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|value| {
            value
                .trim_matches(|char: char| !char.is_alphanumeric() && char != '/' && char != '.')
                .to_lowercase()
        })
        .filter(|value| !value.is_empty())
        .collect()
}

fn truncate_words(value: &str, max_words: usize) -> String {
    let words = value.split_whitespace().collect::<Vec<_>>();
    if words.len() <= max_words {
        value.trim().to_string()
    } else {
        format!("{}...", words[..max_words].join(" "))
    }
}
