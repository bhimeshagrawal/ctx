import { defineGroup } from "@bunli/core";
import addCommand from "./add.js";
import searchCommand from "./search.js";

export default defineGroup({
  name: "memory",
  description: "Manage local memory",
  commands: [addCommand, searchCommand]
});
