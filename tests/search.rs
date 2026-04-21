use std::collections::HashMap;

use ctx::ranking::{rank_results, SearchCandidate};

fn candidate(id: &str, vector_score: f32, keyword_score: f32) -> SearchCandidate {
    SearchCandidate {
        id: id.to_string(),
        kind: "chunk".to_string(),
        document_id: "doc-1".to_string(),
        title: Some("doc".to_string()),
        summary: Some("alpha beta".to_string()),
        memory_type: None,
        scope: None,
        status: None,
        source_path: Some("/tmp/doc.md".to_string()),
        source_refs: vec!["chunk-1".to_string()],
        tags: vec!["notes".to_string()],
        entity_refs: Vec::new(),
        content: "alpha beta gamma".to_string(),
        confidence: 0.0,
        importance: 0.0,
        access_count: 0,
        created_at: None,
        updated_at: None,
        occurred_at: None,
        last_accessed_at: None,
        vector_score,
        keyword_score,
        final_score: 0.0,
    }
}

#[test]
fn hybrid_ranking_merges_vector_and_keyword_scores() {
    let mut keyword = HashMap::new();
    keyword.insert("a".to_string(), 1.0);
    let ranked = rank_results(vec![candidate("a", 0.8, 0.0)], keyword, 5, 0.7, 0.3);
    assert_eq!(ranked.len(), 1);
    assert!((ranked[0].final_score - 0.86).abs() < 0.001);
}
