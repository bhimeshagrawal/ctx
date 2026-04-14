import type { CtxDatabase } from "../storage/lancedb.js";
import { ChunksRepository } from "../storage/repositories/chunks-repo.js";
import type { ChunkRow } from "../types/chunk.js";

export async function vectorSearch(
  db: CtxDatabase,
  queryVector: number[],
  limit: number,
  tags: string[]
): Promise<Map<string, { chunk: ChunkRow; score: number }>> {
  const rows = await new ChunksRepository(db.chunks).listAll();
  const filtered = rows.filter((row) => tags.length === 0 || tags.every((tag) => row.tags.includes(tag)));
  const ranked = filtered
    .map((row) => ({
      chunk: row,
      score: cosineSimilarity(queryVector, decodeVector(row.vectorJson))
    }))
    .sort((left, right) => right.score - left.score)
    .slice(0, limit);

  return new Map(ranked.map((entry) => [entry.chunk.id, entry]));
}

function cosineSimilarity(left: number[], right: number[]): number {
  let dot = 0;
  let leftNorm = 0;
  let rightNorm = 0;
  const size = Math.min(left.length, right.length);

  for (let index = 0; index < size; index += 1) {
    const leftValue = left[index] ?? 0;
    const rightValue = right[index] ?? 0;
    dot += leftValue * rightValue;
    leftNorm += leftValue * leftValue;
    rightNorm += rightValue * rightValue;
  }

  if (leftNorm === 0 || rightNorm === 0) {
    return 0;
  }

  return dot / (Math.sqrt(leftNorm) * Math.sqrt(rightNorm));
}

function decodeVector(value: string): number[] {
  try {
    const parsed = JSON.parse(value);
    return Array.isArray(parsed) ? parsed.map((item) => Number(item) || 0) : [];
  } catch {
    return [];
  }
}
