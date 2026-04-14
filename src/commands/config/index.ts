import { defineGroup } from "@bunli/core";
import showCommand from "./show.js";

export default defineGroup({
  name: "config",
  description: "Inspect ctx configuration",
  commands: [showCommand]
});
