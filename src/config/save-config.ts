import { mkdir, writeFile } from "node:fs/promises";
import type { CtxPaths } from "./paths.js";
import type { CtxConfig } from "./schema.js";

export async function saveConfig(paths: CtxPaths, config: CtxConfig): Promise<void> {
  await mkdir(paths.rootDir, { recursive: true });
  await writeFile(paths.configPath, JSON.stringify(config, null, 2) + "\n", "utf8");
}
