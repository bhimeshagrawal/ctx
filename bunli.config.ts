import { defineConfig } from "@bunli/core";

export default defineConfig({
  name: "ctx",
  version: "0.1.0",
  commands: {
    directory: "./src/commands"
  },
  build: {
    entry: "./src/index.ts",
    outdir: "./dist",
    targets: ["darwin-arm64", "darwin-x64", "linux-x64", "linux-arm64", "windows-x64"]
  }
});
