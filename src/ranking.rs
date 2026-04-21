use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchCandidate {
    pub id: String,
    pub document_id: String,
    pub title: Option<String>,
    pub source_path: Option<String>,
    pub tags: Vec<String>,
    pub content: String,
    pub vector_score: f32,
    pub keyword_score: f32,
    pub final_score: f32,
}

pub fn rank_results(
    vector_matches: Vec<SearchCandidate>,
    keyword_matches: HashMap<String, f32>,
    top_k: usize,
    vector_weight: f32,
    keyword_weight: f32,
) -> Vec<SearchCandidate> {
    let mut merged = HashMap::new();

    for mut candidate in vector_matches {
        candidate.keyword_score = *keyword_matches.get(&candidate.id).unwrap_or(&0.0);
        candidate.final_score =
            (candidate.vector_score * vector_weight) + (candidate.keyword_score * keyword_weight);
        merged.insert(candidate.id.clone(), candidate);
    }

    let mut values = merged.into_values().collect::<Vec<_>>();
    values.sort_by(|left, right| right.final_score.total_cmp(&left.final_score));
    values.truncate(top_k);
    values
}
