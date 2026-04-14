import type { ChunkRow } from "../types/chunk.js";
import type { DocumentRow } from "../types/document.js";
import type { ProfileRow } from "../types/profile.js";

export function seedDocumentRow(): DocumentRow {
  return {
    id: "__seed__",
    sourceType: "text",
    sourcePath: "",
    sourceHash: "__seed__",
    title: "",
    tags: ["__seed__"],
    createdAt: new Date(0).toISOString(),
    updatedAt: new Date(0).toISOString(),
    metadata: "{}"
  };
}

export function seedChunkRow(dimension: number): ChunkRow {
  return {
    id: "__seed__",
    documentId: "__seed__",
    chunkIndex: 0,
    content: "__seed__",
    contentHash: "__seed__",
    tokenEstimate: 1,
    vector: Float32Array.from({ length: dimension }, () => 0),
    vectorJson: JSON.stringify(Array.from({ length: dimension }, () => 0)),
    title: "",
    sourcePath: "",
    tags: ["__seed__"],
    createdAt: new Date(0).toISOString(),
    metadata: "{}"
  };
}

export function seedProfileRow(): ProfileRow {
  return {
    id: "__seed__",
    name: "__seed__",
    defaultTopK: 5,
    defaultChunkSize: 1200,
    defaultChunkOverlap: 150,
    outputMode: "text",
    embeddingModel: "fast-bge-small-en-v1.5",
    metadata: "{}"
  };
}
