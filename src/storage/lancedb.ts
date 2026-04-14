import * as lancedb from "@lancedb/lancedb";
import { mkdir } from "node:fs/promises";
import type { CtxPaths } from "../config/paths.js";
import type { EmbeddingProvider } from "../embeddings/provider.js";
import { seedChunkRow, seedDocumentRow, seedProfileRow } from "./tables.js";

export type CtxDatabase = {
  connection: lancedb.Connection;
  documents: lancedb.Table;
  chunks: lancedb.Table;
  profiles: lancedb.Table;
};

async function openOrCreateTable(
  connection: lancedb.Connection,
  name: string,
  seedRows: Record<string, unknown>[]
): Promise<lancedb.Table> {
  try {
    return await connection.openTable(name);
  } catch {
    const table = await connection.createTable(name, seedRows);
    await table.delete("id = '__seed__'");
    return table;
  }
}

export async function createDatabase(
  paths: CtxPaths,
  provider: EmbeddingProvider
): Promise<CtxDatabase> {
  await mkdir(paths.dataDir, { recursive: true });
  const dimension = await provider.getDimension();
  const connection = await lancedb.connect(paths.dataDir);
  const documents = await openOrCreateTable(connection, "documents", [seedDocumentRow()]);
  const chunks = await openOrCreateTable(connection, "chunks", [seedChunkRow(dimension)]);
  const profiles = await openOrCreateTable(connection, "profiles", [seedProfileRow()]);

  return { connection, documents, chunks, profiles };
}
