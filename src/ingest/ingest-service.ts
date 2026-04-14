import { createHash, randomUUID } from "node:crypto";
import type { CtxConfig } from "../config/schema.js";
import type { EmbeddingProvider } from "../embeddings/provider.js";
import { chunkText } from "./chunk.js";
import type { InputPayload } from "./read-input.js";
import { normalizeContent } from "./normalize.js";
import type { CtxDatabase } from "../storage/lancedb.js";
import { ChunksRepository } from "../storage/repositories/chunks-repo.js";
import { DocumentsRepository } from "../storage/repositories/documents-repo.js";
import type { ChunkRow } from "../types/chunk.js";
import type { DocumentRow } from "../types/document.js";

type IngestOptions = {
  db: CtxDatabase;
  provider: EmbeddingProvider;
  config: CtxConfig;
  input: InputPayload;
  title: string | null;
  tags: string[];
  chunkSize?: number;
  chunkOverlap?: number;
};

export async function runIngest(options: IngestOptions) {
  const normalized = normalizeContent(options.input.content);
  if (!normalized) {
    throw new Error("Input content is empty after normalization");
  }

  const chunkSize = options.chunkSize ?? options.config.defaults.chunkSize;
  const chunkOverlap = options.chunkOverlap ?? options.config.defaults.chunkOverlap;
  const chunked = chunkText(normalized, chunkSize, chunkOverlap);

  const embeddings = await options.provider.embed(chunked.map((item) => item.content));
  const now = new Date().toISOString();
  const documentId = randomUUID();
  const sourceHash = sha256(normalized);
  const document: DocumentRow = {
    id: documentId,
    sourceType: options.input.sourceType,
    sourcePath: options.input.sourcePath,
    sourceHash,
    title: options.title,
    tags: options.tags,
    createdAt: now,
    updatedAt: now,
    metadata: JSON.stringify({})
  };

  const chunks: ChunkRow[] = chunked.map((item, index) => ({
    id: randomUUID(),
    documentId,
    chunkIndex: item.index,
    content: item.content,
    contentHash: sha256(item.content),
    tokenEstimate: item.tokenEstimate,
    embedding: embeddings[index]!,
    title: options.title,
    sourcePath: options.input.sourcePath,
    tags: options.tags,
    createdAt: now,
    metadata: JSON.stringify({})
  }));

  const documentsRepo = new DocumentsRepository(options.db.documents);
  const chunksRepo = new ChunksRepository(options.db.chunks);
  await documentsRepo.add(document);
  await chunksRepo.addMany(chunks);

  return {
    ok: true,
    documentId,
    chunkCount: chunks.length,
    sourceType: options.input.sourceType,
    title: options.title
  };
}

function sha256(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}
