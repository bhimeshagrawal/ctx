import { defineCommand, option } from "@bunli/core";
import { z } from "zod";
import { createPaths } from "../config/paths.js";
import { loadConfig } from "../config/load-config.js";
import { FastEmbedProvider } from "../embeddings/fastembed-provider.js";
import { runDoctor } from "../doctor/doctor-service.js";
import { renderData } from "../output/render.js";

export default defineCommand({
  name: "doctor",
  description: "Check config, storage, and embedding health",
  options: {
    json: option(z.coerce.boolean().default(false), {
      description: "Print machine-readable output"
    })
  },
  handler: async ({ flags }) => {
    const paths = createPaths();
    const config = await loadConfig(paths);
    const provider = new FastEmbedProvider({
      cacheDir: paths.modelsDir,
      model: config.embeddings.model,
      showDownloadProgress: false
    });

    const result = await runDoctor({ paths, config, provider });
    renderData(result, flags.json);
  }
});
