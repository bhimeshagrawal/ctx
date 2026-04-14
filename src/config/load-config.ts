import { readFile } from "node:fs/promises";
import type { CtxPaths } from "./paths.js";
import { configSchema, type CtxConfig } from "./schema.js";

export async function loadConfig(paths: CtxPaths): Promise<CtxConfig> {
  const raw = await readFile(paths.configPath, "utf8");
  return configSchema.parse(JSON.parse(raw));
}
