# ctx

`ctx` is a local-first Rust CLI for memory ingest and retrieval.

Current V1 scope:
- local setup and uninstall
- text/file/stdin memory ingest
- hybrid retrieval with local embeddings
- no hosted API
- no external database server

## Stack

- Rust
- clap
- LanceDB
- fastembed-rs

Runtime state now uses OS-standard locations instead of `~/.ctx/`.

- macOS data: `~/Library/Application Support/ctx`
- macOS cache: `~/Library/Caches/ctx`
- Linux data: `${XDG_DATA_HOME:-~/.local/share}/ctx`
- Linux cache: `${XDG_CACHE_HOME:-~/.cache}/ctx`
- Windows data: `%AppData%/ctx`
- Windows cache: `%LocalAppData%/ctx/cache`

## Commands

- `ctx setup`
- `ctx uninstall`
- `ctx doctor`
- `ctx config show`
- `ctx memory add`
- `ctx memory search`
- `ctx mcp serve`

## MCP

`ctx` can also run as a local MCP server over `stdio` or HTTP.

Start a stdio server for agent clients that launch the process directly:

```bash
ctx mcp serve --transport stdio
```

Start an HTTP server bound to localhost:

```bash
ctx mcp serve --transport http --host 127.0.0.1 --port 8765
```

Notes:
- HTTP is local-only in V1 and binds to `127.0.0.1` by default
- there is no auth layer yet, so remote exposure is not supported
- the MCP contract is transport-neutral and exposes the same tools, resources, and prompts over both transports

Current MCP tools:
- `memory_add`
- `memory_search`
- `setup_run`
- `doctor_run`
- `config_show`
- `update_run`
- `uninstall_run`

Current MCP resources:
- `ctx://config`
- `ctx://paths`
- `ctx://status`

Current MCP prompts:
- `memory-add-workflow`
- `memory-search-workflow`
- `setup-workflow`

## Install

`ctx` is distributed through Homebrew for stable releases.

```bash
brew tap bhimeshagrawal/homebrew-tap
brew install ctx
```

After install:

```bash
ctx setup
```

## Local Development

Requirements:
- Rust toolchain
- `protoc` for LanceDB dependencies

Run checks:

```bash
cargo test
```

Run the CLI locally:

```bash
cargo run -- --help
```

## Release

Stable releases are published from version tags such as `v0.1.0`.

The release workflow will:
- verify that the pushed tag matches `Cargo.toml`
- install the Rust toolchain
- run the Rust test suite
- build the macOS ARM release artifact
- publish a versioned GitHub Release with `checksums.txt`
- update the Homebrew tap formula automatically

Current release asset:
- `ctx-darwin-arm64.tar.gz`

## Smoke Test

```bash
ctx setup
ctx memory add --text "ctx smoke test content about lancedb and fastembed" --tag smoke
ctx memory search "lancedb fastembed" --tag smoke
```

## Uninstall

`ctx uninstall` preserves stored memory by default.

It removes disposable cache state and leaves durable app data intact unless you ask to purge data explicitly.

Destructive purge:

```bash
ctx uninstall --purge-data --force
```

## Upgrade

Upgrade through Homebrew:

```bash
brew upgrade ctx
```

Notes:
- stable installation and upgrades are managed through Homebrew
- the current Homebrew package targets macOS ARM
