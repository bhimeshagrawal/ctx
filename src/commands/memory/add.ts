import { defineCommand, option } from "@bunli/core";
import { z } from "zod";
import { createPaths } from "../../config/paths.js";
import { loadConfig } from "../../config/load-config.js";
import { FastEmbedProvider } from "../../embeddings/fastembed-provider.js";
import { runIngest } from "../../ingest/ingest-service.js";
import { readInput } from "../../ingest/read-input.js";
import { createDatabase } from "../../storage/lancedb.js";
import { renderData } from "../../output/render.js";

export default defineCommand({
  name: "add",
  description: "Add text, files, or stdin content to memory",
  options: {
    file: option(z.string().optional(), { description: "Path to a file to ingest" }),
    text: option(z.string().optional(), { description: "Direct text to ingest" }),
    stdin: option(z.coerce.boolean().default(false), { description: "Read content from stdin" }),
    title: option(z.string().optional(), { description: "Optional document title" }),
    tag: option(z.array(z.string()).default([]), { description: "Tags to attach to the document" }),
    chunkSize: option(z.coerce.number().int().positive().optional(), {
      description: "Chunk size in characters"
    }),
    chunkOverlap: option(z.coerce.number().int().min(0).optional(), {
      description: "Chunk overlap in characters"
    }),
    json: option(z.coerce.boolean().default(false), { description: "Print machine-readable output" })
  },
  handler: async ({ flags }) => {
    const paths = createPaths();
    const config = await loadConfig(paths);
    const input = await readInput({
      file: flags.file,
      text: flags.text,
      stdin: flags.stdin
    });
    const provider = new FastEmbedProvider({
      cacheDir: paths.modelsDir,
      model: config.embeddings.model,
      showDownloadProgress: false
    });
    const db = await createDatabase(paths, provider);
    const result = await runIngest({
      db,
      provider,
      config,
      input,
      title: flags.title ?? input.title,
      tags: flags.tag,
      chunkSize: flags.chunkSize,
      chunkOverlap: flags.chunkOverlap
    });
    renderData(result, flags.json);
  }
});
