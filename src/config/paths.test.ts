import { expect, test } from "bun:test";
import { createPaths } from "./paths.js";

test("createPaths uses the provided base directory", () => {
  const paths = createPaths("/tmp/ctx-test");
  expect(paths.modelsDir).toBe("/tmp/ctx-test/models");
  expect(paths.configPath).toBe("/tmp/ctx-test/config.json");
});
