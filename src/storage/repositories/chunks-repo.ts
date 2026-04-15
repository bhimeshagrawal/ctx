import type { Table } from "@lancedb/lancedb";
import type { ChunkRow } from "../../types/chunk.js";

function normalizeChunkRow(row: unknown): ChunkRow {
  const r = row as Record<string, unknown>;
  return {
    ...(r as ChunkRow),
    tags: Array.from((r.tags as Iterable<string> | null | undefined) ?? [])
  };
}

export class ChunksRepository {
  constructor(private readonly table: Table) {}

  async addMany(rows: ChunkRow[]): Promise<void> {
    if (rows.length === 0) {
      return;
    }
    await this.table.add(rows);
  }

  async vectorSearch(vector: number[], limit: number): Promise<ChunkRow[]> {
    const rows = await this.table.search(vector).limit(limit).toArray();
    return rows.map(normalizeChunkRow);
  }

  async listAll(): Promise<ChunkRow[]> {
    const count = await this.table.countRows();
    if (count === 0) {
      return [];
    }
    const rows = await this.table.query().limit(count).toArray();
    return rows.map(normalizeChunkRow);
  }
}
