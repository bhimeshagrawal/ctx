import { expect, test } from "bun:test";
import { parseTags } from "./tags.js";

test("parseTags splits comma-separated values", () => {
  expect(parseTags("one, two,three")).toEqual(["one", "two", "three"]);
  expect(parseTags()).toEqual([]);
});
