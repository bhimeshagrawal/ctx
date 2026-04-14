import { defineCommand, option } from "@bunli/core";
import { z } from "zod";
import { createPaths } from "../../config/paths.js";
import { loadConfig } from "../../config/load-config.js";
import { FastEmbedProvider } from "../../embeddings/fastembed-provider.js";
import { renderData } from "../../output/render.js";
import { runSearch } from "../../retrieval/search-service.js";
import { createDatabase } from "../../storage/lancedb.js";

export default defineCommand({
  name: "search",
  description: "Search local memory using vector and keyword retrieval",
  options: {
    topK: option(z.coerce.number().int().positive().default(5), {
      description: "Number of results to return"
    }),
    tag: option(z.array(z.string()).default([]), { description: "Filter by tags" }),
    json: option(z.coerce.boolean().default(false), {
      description: "Print machine-readable output"
    })
  },
  handler: async ({ flags, positional }) => {
    const query = positional.join(" ").trim();
    if (!query) {
      throw new Error("Search query is required");
    }

    const paths = createPaths();
    const config = await loadConfig(paths);
    const provider = new FastEmbedProvider({
      cacheDir: paths.modelsDir,
      model: config.embeddings.model,
      showDownloadProgress: false
    });
    const db = await createDatabase(paths, provider);
    const result = await runSearch({
      db,
      provider,
      query,
      topK: flags.topK,
      tags: flags.tag
    });
    renderData(result, flags.json);
  }
});
