use anyhow::{anyhow, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Chunk {
    pub index: usize,
    pub content: String,
    pub token_estimate: usize,
}

pub fn chunk_text(input: &str, size: usize, overlap: usize) -> Result<Vec<Chunk>> {
    if overlap >= size {
        return Err(anyhow!("chunk overlap must be smaller than chunk size"));
    }
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let chars: Vec<char> = input.chars().collect();
    let mut cursor = 0;
    let mut chunks = Vec::new();
    let mut index = 0;

    while cursor < chars.len() {
        let end = usize::min(cursor + size, chars.len());
        let content: String = chars[cursor..end].iter().collect();
        chunks.push(Chunk {
            index,
            token_estimate: content.split_whitespace().count().max(1),
            content,
        });
        if end == chars.len() {
            break;
        }
        cursor = end.saturating_sub(overlap);
        index += 1;
    }

    Ok(chunks)
}
