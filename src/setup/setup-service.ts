import { mkdir } from "node:fs/promises";
import type { CtxPaths } from "../config/paths.js";
import type { CtxConfig } from "../config/schema.js";
import type { EmbeddingProvider } from "../embeddings/provider.js";
import { createDatabase } from "../storage/lancedb.js";
import { ProfilesRepository } from "../storage/repositories/profiles-repo.js";

type SetupOptions = {
  paths: CtxPaths;
  config: CtxConfig;
  provider: EmbeddingProvider;
};

export async function ensureSetup(options: SetupOptions) {
  await mkdir(options.paths.rootDir, { recursive: true });
  await mkdir(options.paths.dataDir, { recursive: true });
  await mkdir(options.paths.modelsDir, { recursive: true });
  await mkdir(options.paths.logsDir, { recursive: true });
  await mkdir(options.paths.tmpDir, { recursive: true });

  await options.provider.init();
  const db = await createDatabase(options.paths, options.provider);
  const profilesRepo = new ProfilesRepository(db.profiles);
  await profilesRepo.ensureDefault({
    id: "default",
    name: "default",
    defaultTopK: options.config.defaults.topK,
    defaultChunkSize: options.config.defaults.chunkSize,
    defaultChunkOverlap: options.config.defaults.chunkOverlap,
    outputMode: options.config.defaults.outputMode,
    embeddingModel: options.config.embeddings.model,
    metadata: JSON.stringify({})
  });

  const sampleVector = await options.provider.embedQuery("ctx setup health check");
  return {
    ok: true,
    paths: options.paths,
    embeddingModel: options.config.embeddings.model,
    embeddingDimension: sampleVector.length
  };
}
