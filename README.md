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
