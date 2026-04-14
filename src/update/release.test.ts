import { expect, test } from "bun:test";
import { releaseAssetNames, releaseBaseUrl } from "./release.js";

test("releaseBaseUrl uses latest by default", () => {
  expect(releaseBaseUrl("owner/repo")).toBe("https://github.com/owner/repo/releases/latest/download");
  expect(releaseBaseUrl("owner/repo", "v1.2.3")).toBe(
    "https://github.com/owner/repo/releases/download/v1.2.3"
  );
});

test("releaseAssetNames matches install script naming", () => {
  expect(releaseAssetNames({ os: "darwin", arch: "arm64" }).archive).toBe("ctx-darwin-arm64.tar.gz");
  expect(releaseAssetNames({ os: "linux", arch: "x64" }).archive).toBe("ctx-linux-x64.tar.gz");
});
