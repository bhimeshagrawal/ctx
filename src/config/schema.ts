import type { CtxPaths } from "./paths.js";
import { z } from "zod";

export const configSchema = z.object({
  version: z.literal(1),
  paths: z.object({
    rootDir: z.string(),
    dataDir: z.string(),
    modelsDir: z.string(),
    logsDir: z.string(),
    tmpDir: z.string()
  }),
  defaults: z.object({
    topK: z.number().int().positive(),
    chunkSize: z.number().int().positive(),
    chunkOverlap: z.number().int().min(0),
    outputMode: z.enum(["text", "json"])
  }),
  embeddings: z.object({
    provider: z.literal("fastembed"),
    model: z.string()
  })
});

export type CtxConfig = z.infer<typeof configSchema>;

export function createDefaultConfig(paths: CtxPaths): CtxConfig {
  return {
    version: 1,
    paths: {
      rootDir: paths.rootDir,
      dataDir: paths.dataDir,
      modelsDir: paths.modelsDir,
      logsDir: paths.logsDir,
      tmpDir: paths.tmpDir
    },
    defaults: {
      topK: 5,
      chunkSize: 1200,
      chunkOverlap: 150,
      outputMode: "text"
    },
    embeddings: {
      provider: "fastembed",
      model: "fast-bge-small-en-v1.5"
    }
  };
}
