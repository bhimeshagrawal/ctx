# ctx

`ctx` is a local-first CLI for memory ingest and retrieval.

Current V1 scope:
- local setup and uninstall
- text/file/stdin memory ingest
- hybrid retrieval with local embeddings
- no hosted API
- no external database server

## Stack

- Bun + Bunli
- LanceDB
- FastEmbed

All runtime state lives under `~/.ctx/`.

## Commands

- `ctx setup`
- `ctx update`
- `ctx uninstall`
- `ctx doctor`
- `ctx config show`
- `ctx memory add`
- `ctx memory search`

## Install

The current distribution path is a direct install script over GitHub Releases.

```bash
curl -fsSL https://raw.githubusercontent.com/bhimeshagrawal/ctx/main/install.sh | bash
```

Defaults:
- installs the binary into `~/.local/bin`
- prints a `PATH` hint if needed
- does not run setup automatically unless `CTX_RUN_SETUP=1`

Optional environment variables:
- `CTX_INSTALL_DIR` to change the install directory
- `CTX_VERSION` to pin a release tag instead of `latest`
- `CTX_RUN_SETUP=1` to run `ctx setup` after install
- `CTX_REPO` to override the GitHub repository slug

Example:

```bash
CTX_INSTALL_DIR="$HOME/bin" CTX_RUN_SETUP=1 \
curl -fsSL https://raw.githubusercontent.com/bhimeshagrawal/ctx/main/install.sh | bash
```

## Local Development

Requirements:
- Bun

Install dependencies:

```bash
bun install
```

Run checks:

```bash
bun test
bun run typecheck
```

Run the CLI locally:

```bash
bun ./src/index.ts --help
```

## Release

Releases are published automatically on every push to `main` through GitHub Actions.

The release workflow will:
- install Bun dependencies for all target platforms
- run tests and typecheck
- build all standalone binaries
- publish release assets and `checksums.txt`

The rolling release tag is `latest`.

Expected release assets:
- `ctx-darwin-arm64.tar.gz`
- `ctx-darwin-x64.tar.gz`
- `ctx-linux-arm64.tar.gz`
- `ctx-linux-x64.tar.gz`
- `ctx-windows-x64.zip`

## Smoke Test

```bash
ctx setup
ctx memory add --text "ctx smoke test content about lancedb and fastembed" --tag smoke
ctx memory search "lancedb fastembed" --tag smoke
```

## Uninstall

`ctx uninstall` removes app-managed local state under `~/.ctx/`.

It does not remove:
- the installed binary
- package-manager metadata

Non-interactive cleanup:

```bash
ctx uninstall --force
```

## Update

Once installed as a compiled binary, update in place with:

```bash
ctx update
```

You can also target a specific release:

```bash
ctx update --version v0.1.0
```

Notes:
- `ctx update` is intended for installed binaries, not `bun ./src/index.ts`
- it currently supports macOS and Linux
- it replaces the existing binary in place after checksum verification
