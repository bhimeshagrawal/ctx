import type { ChunkRow, SearchCandidate } from "../types/chunk.js";

export function rankResults(args: {
  vectorMatches: Map<string, { chunk: ChunkRow; score: number }>;
  keywordMatches: Map<string, number>;
  topK: number;
}): SearchCandidate[] {
  const merged = new Map<string, SearchCandidate>();

  for (const [id, match] of args.vectorMatches.entries()) {
    merged.set(id, {
      ...match.chunk,
      vectorScore: match.score,
      keywordScore: args.keywordMatches.get(id) ?? 0,
      finalScore: 0
    });
  }

  for (const [id, score] of args.keywordMatches.entries()) {
    const existing = merged.get(id);
    if (existing) {
      existing.keywordScore = score;
      continue;
    }
  }

  for (const candidate of merged.values()) {
    candidate.finalScore = candidate.vectorScore * 0.7 + candidate.keywordScore * 0.3;
  }

  return [...merged.values()]
    .sort((left, right) => right.finalScore - left.finalScore)
    .slice(0, args.topK);
}
