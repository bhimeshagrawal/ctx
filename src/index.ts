import { createCLI } from "@bunli/core";
import configGroup from "./commands/config/index.js";
import doctorCommand from "./commands/doctor.js";
import memoryGroup from "./commands/memory/index.js";
import setupCommand from "./commands/setup.js";
import uninstallCommand from "./commands/uninstall.js";
import updateCommand from "./commands/update.js";

const cli = await createCLI();

cli.command(setupCommand);
cli.command(uninstallCommand);
cli.command(doctorCommand);
cli.command(updateCommand);
cli.command(configGroup);
cli.command(memoryGroup);

await cli.init();
await cli.run();
