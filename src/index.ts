import { createCLI } from "@bunli/core";
import { tmpdir } from "node:os";
import path from "node:path";
import configGroup from "./commands/config/index.js";
import doctorCommand from "./commands/doctor.js";
import memoryGroup from "./commands/memory/index.js";
import setupCommand from "./commands/setup.js";
import uninstallCommand from "./commands/uninstall.js";
import updateCommand from "./commands/update.js";

// Compiled binaries cannot dynamically import TypeScript files. If the binary
// is run from the project directory bunli finds bunli.config.ts and fails to
// load it (ConfigLoadError). Temporarily chdir to tmpdir so bunli sees no
// config file and falls through to the inline config we provide.
const isCompiledBinary = !path.basename(process.execPath).toLowerCase().startsWith("bun");
const originalCwd = process.cwd();
if (isCompiledBinary) {
  process.chdir(tmpdir());
}

const cli = await createCLI({ name: "ctx", version: "0.1.0" });

if (isCompiledBinary) {
  process.chdir(originalCwd);
}

cli.command(setupCommand);
cli.command(uninstallCommand);
cli.command(doctorCommand);
cli.command(updateCommand);
cli.command(configGroup);
cli.command(memoryGroup);

await cli.init();
await cli.run();
