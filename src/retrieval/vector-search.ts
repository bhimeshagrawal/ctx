import type { CtxDatabase } from "../storage/lancedb.js";
import { ChunksRepository } from "../storage/repositories/chunks-repo.js";
import type { ChunkRow } from "../types/chunk.js";

export async function vectorSearch(
  db: CtxDatabase,
  queryVector: number[],
  limit: number,
  tags: string[]
): Promise<Map<string, { chunk: ChunkRow; score: number }>> {
  const rows = await new ChunksRepository(db.chunks).vectorSearch(queryVector, limit);
  const filtered = rows.filter((row) => tags.length === 0 || tags.every((tag) => row.tags.includes(tag)));
  const results = new Map<string, { chunk: ChunkRow; score: number }>();
  const total = Math.max(filtered.length - 1, 1);
  filtered.forEach((row, index) => {
    results.set(row.id, {
      chunk: row,
      score: Math.max(0, 1 - index / total)
    });
  });
  return results;
}
