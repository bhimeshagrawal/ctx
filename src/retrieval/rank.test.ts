import { expect, test } from "bun:test";
import { rankResults } from "./rank.js";
import type { ChunkRow } from "../types/chunk.js";

const baseChunk: ChunkRow = {
  id: "a",
  documentId: "doc-1",
  chunkIndex: 0,
  content: "alpha beta gamma",
  contentHash: "hash",
  tokenEstimate: 3,
  vector: [0.1, 0.2],
  vectorJson: "[0.1,0.2]",
  title: "doc",
  sourcePath: "/tmp/doc.md",
  tags: ["notes"],
  createdAt: new Date().toISOString(),
  metadata: "{}"
};

test("rankResults merges vector and keyword scores", () => {
  const ranked = rankResults({
    vectorMatches: new Map([
      ["a", { chunk: baseChunk, score: 0.8 }]
    ]),
    keywordMatches: new Map([["a", 1]]),
    topK: 5
  });

  expect(ranked).toHaveLength(1);
  expect(ranked[0]?.finalScore).toBeCloseTo(0.86);
});
