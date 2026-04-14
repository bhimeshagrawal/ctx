import { platform, arch } from "node:process";

export function detectPlatformTarget(): { os: "darwin" | "linux"; arch: "arm64" | "x64" } {
  const os = platform === "darwin" || platform === "linux" ? platform : null;
  if (!os) {
    throw new Error(`Unsupported operating system for self-update: ${platform}`);
  }

  const cpu = arch === "arm64" ? "arm64" : arch === "x64" ? "x64" : null;
  if (!cpu) {
    throw new Error(`Unsupported architecture for self-update: ${arch}`);
  }

  return { os, arch: cpu };
}

export function releaseBaseUrl(repository: string, version?: string): string {
  if (version && version !== "latest") {
    return `https://github.com/${repository}/releases/download/${version}`;
  }
  return `https://github.com/${repository}/releases/latest/download`;
}

export function releaseAssetNames(target: { os: "darwin" | "linux"; arch: "arm64" | "x64" }) {
  return {
    archive: `ctx-${target.os}-${target.arch}.tar.gz`,
    checksums: "checksums.txt"
  };
}
