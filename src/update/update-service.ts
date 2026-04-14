import { chmod, mkdtemp, readFile, rename, rm, writeFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { releaseAssetNames, releaseBaseUrl, detectPlatformTarget } from "./release.js";

type UpdateOptions = {
  version?: string;
  repository: string;
};

export async function runUpdate(options: UpdateOptions) {
  const execPath = process.execPath;
  const execName = path.basename(execPath).toLowerCase();

  if (execName === "bun" || execName === "bun.exe") {
    throw new Error("ctx update must be run from an installed ctx binary, not via `bun ./src/index.ts`");
  }

  const target = detectPlatformTarget();
  const assets = releaseAssetNames(target);
  const baseUrl = releaseBaseUrl(options.repository, options.version);
  const tmpRoot = await mkdtemp(path.join(tmpdir(), "ctx-update-"));

  try {
    const archivePath = path.join(tmpRoot, assets.archive);
    const checksumsPath = path.join(tmpRoot, assets.checksums);
    const extractDir = path.join(tmpRoot, "extract");

    await downloadToFile(`${baseUrl}/${assets.archive}`, archivePath);
    await downloadToFile(`${baseUrl}/${assets.checksums}`, checksumsPath);
    await verifyChecksum(archivePath, checksumsPath, assets.archive);

    await Bun.$`mkdir -p ${extractDir}`.quiet();
    await Bun.$`tar -xzf ${archivePath} -C ${extractDir}`.quiet();

    const extractedBinary = path.join(extractDir, "ctx");
    const binaryBuffer = await readFile(extractedBinary);
    const stagingPath = `${execPath}.new`;
    await writeFile(stagingPath, binaryBuffer);
    await chmod(stagingPath, 0o755);
    await rename(stagingPath, execPath);

    return {
      ok: true,
      updated: true,
      binaryPath: execPath,
      archive: assets.archive,
      repository: options.repository,
      channel: options.version ?? "latest"
    };
  } finally {
    await rm(tmpRoot, { recursive: true, force: true });
  }
}

async function downloadToFile(url: string, filePath: string): Promise<void> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Download failed for ${url}: ${response.status} ${response.statusText}`);
  }
  const bytes = Buffer.from(await response.arrayBuffer());
  await writeFile(filePath, bytes);
}

async function verifyChecksum(archivePath: string, checksumsPath: string, assetName: string): Promise<void> {
  const checksums = await readFile(checksumsPath, "utf8");
  const expectedLine = checksums
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => line.endsWith(` ${assetName}`));

  if (!expectedLine) {
    throw new Error(`No checksum found for ${assetName}`);
  }

  const expected = expectedLine.split(/\s+/)[0];
  const actual = createHash("sha256").update(await readFile(archivePath)).digest("hex");
  if (expected !== actual) {
    throw new Error(`Checksum mismatch for ${assetName}`);
  }
}
