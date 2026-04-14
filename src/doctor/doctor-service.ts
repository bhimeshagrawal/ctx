import { access, constants, stat } from "node:fs/promises";
import type { CtxPaths } from "../config/paths.js";
import type { CtxConfig } from "../config/schema.js";
import type { EmbeddingProvider } from "../embeddings/provider.js";
import { createDatabase } from "../storage/lancedb.js";

type DoctorOptions = {
  paths: CtxPaths;
  config: CtxConfig;
  provider: EmbeddingProvider;
};

export async function runDoctor(options: DoctorOptions) {
  const checks = [];

  checks.push(await writableCheck(options.paths.rootDir));
  checks.push(await writableCheck(options.paths.dataDir));
  checks.push(await writableCheck(options.paths.modelsDir));

  const providerStatus = await options.provider.healthCheck();
  checks.push({
    name: "embeddings",
    ok: providerStatus.ok,
    detail: providerStatus.detail
  });

  const db = await createDatabase(options.paths, options.provider);
  checks.push({
    name: "storage",
    ok: Boolean(db.documents && db.chunks && db.profiles),
    detail: "tables-ready"
  });

  return {
    ok: checks.every((check) => check.ok),
    configVersion: options.config.version,
    checks
  };
}

async function writableCheck(targetPath: string) {
  try {
    await stat(targetPath);
    await access(targetPath, constants.W_OK);
    return { name: targetPath, ok: true, detail: "writable" };
  } catch (error) {
    return {
      name: targetPath,
      ok: false,
      detail: error instanceof Error ? error.message : String(error)
    };
  }
}
