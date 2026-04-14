import { homedir } from "node:os";
import path from "node:path";

export type CtxPaths = {
  rootDir: string;
  dataDir: string;
  modelsDir: string;
  logsDir: string;
  tmpDir: string;
  configPath: string;
};

export function createPaths(baseDir = path.join(homedir(), ".ctx")): CtxPaths {
  return {
    rootDir: baseDir,
    dataDir: path.join(baseDir, "data"),
    modelsDir: path.join(baseDir, "models"),
    logsDir: path.join(baseDir, "logs"),
    tmpDir: path.join(baseDir, "tmp"),
    configPath: path.join(baseDir, "config.json")
  };
}
