import { defineCommand, option } from "@bunli/core";
import { z } from "zod";
import { createPaths } from "../config/paths.js";
import { renderData } from "../output/render.js";
import { runUninstall } from "../setup/uninstall-service.js";

export default defineCommand({
  name: "uninstall",
  description: "Remove app-managed local state",
  options: {
    force: option(z.coerce.boolean().default(false), {
      description: "Skip confirmation"
    }),
    json: option(z.coerce.boolean().default(false), {
      description: "Print machine-readable output"
    })
  },
  handler: async ({ flags, prompt }) => {
    const paths = createPaths();
    const targetSummary = {
      rootDir: paths.rootDir,
      configPath: paths.configPath,
      dataDir: paths.dataDir,
      modelsDir: paths.modelsDir,
      logsDir: paths.logsDir,
      tmpDir: paths.tmpDir
    };

    if (!flags.force) {
      const confirmed = await prompt.confirm(
        `Delete ctx local data under ${paths.rootDir}? This will not remove the installed binary.`,
        { default: false }
      );
      if (!confirmed) {
        renderData({ ok: false, removed: [], skipped: ["cancelled"] }, flags.json);
        return;
      }
    }

    const result = await runUninstall(paths);
    renderData({ ...result, targets: targetSummary }, flags.json);
  }
});
