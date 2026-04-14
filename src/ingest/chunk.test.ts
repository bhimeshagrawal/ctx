import { expect, test } from "bun:test";
import { chunkText } from "./chunk.js";

test("chunkText splits content using size and overlap", () => {
  const chunks = chunkText("abcdefghij".repeat(30), 50, 10);
  expect(chunks.length).toBeGreaterThan(1);
  expect(chunks[0]?.content.length).toBeLessThanOrEqual(50);
});

test("chunkText rejects overlap greater than size", () => {
  expect(() => chunkText("hello", 10, 10)).toThrow();
});
