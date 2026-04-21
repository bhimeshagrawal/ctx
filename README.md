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
- `ctx update`
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
- `CTX_DATA_DIR` to override the durable app-data root
- `CTX_CACHE_DIR` to override the cache root

Example:

```bash
CTX_INSTALL_DIR="$HOME/bin" CTX_RUN_SETUP=1 \
curl -fsSL https://raw.githubusercontent.com/bhimeshagrawal/ctx/main/install.sh | bash
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

Releases are published automatically on every push to `main` through GitHub Actions.

The release workflow will:
- install the Rust toolchain
- run the Rust test suite
- build standalone release binaries
- publish release assets and `checksums.txt`

The rolling release tag is `latest`.

Current release assets:
- `ctx-darwin-arm64.tar.gz`
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

`ctx uninstall` preserves stored memory by default.

It removes disposable cache state and leaves durable app data intact unless you ask to purge data explicitly.

Destructive purge:

```bash
ctx uninstall --purge-data --force
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
- `ctx update` is intended for installed binaries, not `cargo run`
- it currently supports macOS and Linux
- it replaces the existing binary in place after checksum verification
