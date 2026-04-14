import { rm } from "node:fs/promises";
import path from "node:path";
import type { CtxPaths } from "../config/paths.js";

export async function runUninstall(paths: CtxPaths) {
  const targets = [
    paths.configPath,
    paths.dataDir,
    paths.modelsDir,
    paths.logsDir,
    paths.tmpDir
  ];

  const removed: string[] = [];
  const skipped: string[] = [];

  for (const target of targets) {
    if (!target.startsWith(paths.rootDir + path.sep) && target !== paths.configPath) {
      throw new Error(`Refusing to remove path outside app root: ${target}`);
    }

    try {
      await rm(target, { recursive: true, force: true });
      removed.push(target);
    } catch {
      skipped.push(target);
    }
  }

  return { ok: true, removed, skipped };
}
