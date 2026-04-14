import { defineCommand, option } from "@bunli/core";
import { z } from "zod";
import { renderData } from "../output/render.js";
import { runUpdate } from "../update/update-service.js";

export default defineCommand({
  name: "update",
  description: "Update the installed ctx binary from GitHub Releases",
  options: {
    version: option(z.string().optional(), {
      description: "Release version to install; defaults to latest"
    }),
    force: option(z.coerce.boolean().default(false), {
      description: "Skip confirmation"
    }),
    json: option(z.coerce.boolean().default(false), {
      description: "Print machine-readable output"
    })
  },
  handler: async ({ flags, prompt }) => {
    if (!flags.force) {
      const confirmed = await prompt.confirm(
        `Download and replace the current ctx binary${flags.version ? ` with ${flags.version}` : ""}?`,
        { default: true }
      );
      if (!confirmed) {
        renderData({ ok: false, skipped: true }, flags.json);
        return;
      }
    }

    const result = await runUpdate({
      version: flags.version,
      repository: process.env.CTX_REPO ?? "bhimeshagrawal/ctx"
    });
    renderData(result, flags.json);
  }
});
