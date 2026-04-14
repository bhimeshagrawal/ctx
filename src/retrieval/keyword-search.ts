import type { ChunkRow } from "../types/chunk.js";

export function keywordSearch(chunks: ChunkRow[], query: string, tags: string[]): Map<string, number> {
  const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
  const results = new Map<string, number>();

  for (const chunk of chunks) {
    if (tags.length > 0 && !tags.every((tag) => chunk.tags.includes(tag))) {
      continue;
    }

    const haystack = chunk.content.toLowerCase();
    let score = 0;
    for (const term of terms) {
      if (haystack.includes(term)) {
        score += 1;
      }
    }

    if (score > 0) {
      results.set(chunk.id, score / terms.length);
    }
  }

  return results;
}
