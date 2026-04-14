import * as lancedb from "@lancedb/lancedb";
import type * as arrow from "apache-arrow";
import { mkdir } from "node:fs/promises";
import type { CtxPaths } from "../config/paths.js";
import type { EmbeddingProvider } from "../embeddings/provider.js";
import { createChunksSchema, createDocumentsSchema, createProfilesSchema } from "./tables.js";

export type CtxDatabase = {
  connection: lancedb.Connection;
  documents: lancedb.Table;
  chunks: lancedb.Table;
  profiles: lancedb.Table;
};

async function openOrCreateTable(
  connection: lancedb.Connection,
  name: string,
  schemaFactory: () => arrow.Schema
): Promise<lancedb.Table> {
  try {
    return await connection.openTable(name);
  } catch {
    return connection.createEmptyTable(name, schemaFactory());
  }
}

export async function createDatabase(
  paths: CtxPaths,
  provider: EmbeddingProvider
): Promise<CtxDatabase> {
  await mkdir(paths.dataDir, { recursive: true });
  const dimension = await provider.getDimension();
  const connection = await lancedb.connect(paths.dataDir);
  const documents = await openOrCreateTable(connection, "documents", createDocumentsSchema);
  const chunks = await openOrCreateTable(connection, "chunks", () => createChunksSchema(dimension));
  const profiles = await openOrCreateTable(connection, "profiles", createProfilesSchema);

  return { connection, documents, chunks, profiles };
}
