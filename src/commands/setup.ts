import { defineCommand } from "@bunli/core";
import { z } from "zod";
import { option } from "@bunli/core";
import { ensureSetup } from "../setup/setup-service.js";
import { createPaths } from "../config/paths.js";
import { loadConfig } from "../config/load-config.js";
import { saveConfig } from "../config/save-config.js";
import { createDefaultConfig } from "../config/schema.js";
import { FastEmbedProvider } from "../embeddings/fastembed-provider.js";
import { renderData } from "../output/render.js";

export default defineCommand({
  name: "setup",
  description: "Initialize local directories, config, storage, and embeddings",
  options: {
    json: option(z.coerce.boolean().default(false), {
      description: "Print machine-readable output"
    }),
    force: option(z.coerce.boolean().default(false), {
      description: "Overwrite config defaults if needed"
    })
  },
  handler: async ({ flags }) => {
    const paths = createPaths();
    const existingConfig = await loadConfig(paths).catch(() => null);
    const config = existingConfig ?? createDefaultConfig(paths);

    if (flags.force || existingConfig === null) {
      await saveConfig(paths, config);
    }

    const provider = new FastEmbedProvider({
      cacheDir: paths.modelsDir,
      model: config.embeddings.model,
      showDownloadProgress: !flags.json
    });

    const result = await ensureSetup({ paths, config, provider });
    await saveConfig(paths, config);
    renderData(result, flags.json);
  }
});
