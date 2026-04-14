import type { EmbeddingProvider } from "../embeddings/provider.js";
import type { CtxDatabase } from "../storage/lancedb.js";
import { ChunksRepository } from "../storage/repositories/chunks-repo.js";
import { keywordSearch } from "./keyword-search.js";
import { rankResults } from "./rank.js";
import { vectorSearch } from "./vector-search.js";

type SearchOptions = {
  db: CtxDatabase;
  provider: EmbeddingProvider;
  query: string;
  topK: number;
  tags: string[];
};

export async function runSearch(options: SearchOptions) {
  const queryVector = await options.provider.embedQuery(options.query);
  const vectorMatches = await vectorSearch(options.db, queryVector, options.topK * 4, options.tags);
  const allChunks = await new ChunksRepository(options.db.chunks).listAll();
  const keywordMatches = keywordSearch(allChunks, options.query, options.tags);
  const results = rankResults({
    vectorMatches,
    keywordMatches,
    topK: options.topK
  });

  return {
    ok: true,
    query: options.query,
    count: results.length,
    results: results.map(({ vector, vectorJson, ...rest }) => rest)
  };
}
