# AI Memory CLI Plan

## Goal

Build a self-sufficient, local-first CLI in this folder that provides persistent agent memory without requiring a hosted database or paid model APIs.

## Architecture Decision

Use:
- `Bun` + `Bunli` for the CLI
- `LanceDB` for local embedded storage
- `FastEmbed` for local in-process embeddings

Do not use for V1:
- Postgres
- pgvector
- OpenAI SDK
- mandatory external model services

## Why This Stack

### Bun + Bunli

- Fast iteration during development
- Type-safe CLI structure
- Cross-platform standalone builds
- Can compile to a single executable for end users

### LanceDB

- Embedded and file-backed, similar to SQLite in deployment shape
- Good fit for local vector, keyword, and hybrid retrieval
- No separate DB server to install or keep running

### FastEmbed

- Local embeddings with no API cost
- Runs in-process instead of depending on a separate local service
- Avoids setup friction from requiring Ollama just for embeddings

## Product Shape

The CLI should be useful as a standalone local memory tool, not as a general agent framework.

Core concerns:

1. Session memory
2. Knowledge ingestion
3. Retrieval and context assembly
4. Optional model execution

## Scope for V1

### Commands

- `ctx setup`
  - Create local directories, initialize LanceDB, warm embedding model, and write config.
- `ctx uninstall`
  - Remove local app data, config, caches, and indexes with confirmation or `--force`.
- `ctx doctor`
  - Check binary health, config, DB path, model availability, and file permissions.
- `ctx chat`
  - Start or continue a session, store message history, assemble context, and optionally call a local model provider.
- `ctx memory add`
  - Add text, file content, markdown, or stdin to semantic memory.
- `ctx memory search`
  - Search across semantic memory with hybrid retrieval.
- `ctx sessions list`
  - List recent sessions.
- `ctx sessions show <id>`
  - Show one conversation timeline.
- `ctx config show`
  - Print resolved paths and active settings.

### Memory Model

- Episodic memory
  - session metadata
  - timestamped messages
  - tool events
- Semantic memory
  - chunked content
  - embeddings
  - source metadata
  - tags
- Procedural memory
  - user preferences
  - output style
  - default retrieval settings
  - preferred local model configuration

## Storage Layout

Use a user-local app directory, for example:

- macOS/Linux: `~/.ctx/`

Suggested structure:

- `~/.ctx/config.json`
- `~/.ctx/data/`
- `~/.ctx/models/`
- `~/.ctx/logs/`
- `~/.ctx/tmp/`

LanceDB tables live under `~/.ctx/data/`.
Embedding model assets are cached under the model directory or the provider’s own cache path.

## Self-Sufficiency Strategy

### What “self-sufficient” means in V1

- No paid APIs required
- No external database server
- No mandatory background service
- No network needed after initial install and model download
- Local data can be removed cleanly with `ctx uninstall`

### What still requires one-time download

- the CLI binary itself
- the embedding model weights used by `FastEmbed`

After those are present, retrieval and storage can work fully offline.

## Installation Strategy

### End-user installation target

Distribute prebuilt standalone binaries per platform.

Examples:
- macOS Apple Silicon
- macOS Intel
- Linux x64
- Linux ARM64
- Windows x64

Use Bun compiled executables for release builds.

## Distribution Strategy

### Canonical release channel

Use GitHub Releases as the source of truth for all published binaries.

Release artifacts:
- `ctx-darwin-arm64.tar.gz`
- `ctx-darwin-x64.tar.gz`
- `ctx-linux-x64.tar.gz`
- `ctx-linux-arm64.tar.gz`
- `ctx-windows-x64.zip`

Each release should include:
- compiled binary
- checksum file
- short release notes

### Primary install channel

Use Homebrew as the main user-friendly install path for macOS and Linux.

Recommended shape:
- create a dedicated tap, for example `yourorg/homebrew-tap`
- publish a formula for `ctx`
- formula downloads the correct GitHub Release artifact
- install instructions become `brew install yourorg/tap/ctx`

### Secondary install channel

Provide a direct install script for macOS and Linux.

The script should:
- detect OS and architecture
- download the correct GitHub Release asset
- verify checksum
- place the binary in a user-local bin directory
- optionally run `ctx setup`

This should be a convenience layer over GitHub Releases, not a separate distribution system.

### Windows distribution

For V1, distribute Windows binaries through GitHub Releases.

Later options:
- Scoop
- winget

### npm decision

Do not use npm as the primary distribution channel for V1.

Reason:
- this project is intended to ship as a standalone compiled CLI
- npm is better suited to JavaScript package distribution than native-style binary delivery

An npm wrapper can be added later if there is real demand.

### Smooth install flow

#### Option A: best default

1. User downloads one archive for their platform
2. User installs via Homebrew or direct install script
3. User runs `ctx setup`
4. Setup:
   - creates `~/.ctx/`
   - initializes LanceDB tables
   - downloads or warms the local embedding model
   - verifies a test embedding roundtrip
   - offers shell completion install

#### Option B: nicer packaging later

- Homebrew formula for macOS/Linux
- Scoop or winget for Windows

## First-Run Experience

`ctx setup` should do almost everything.

Expected behavior:

1. Detect OS, arch, writable directories
2. Create app folders
3. Initialize config with sane defaults
4. Initialize LanceDB schema
5. Download or initialize the default embedding model
6. Run smoke tests:
   - create sample record
   - generate one embedding
   - execute one retrieval
7. Print success state and paths

## Uninstall Strategy

`ctx uninstall` should remove local runtime state created by the app, but should not try to remove the installed binary or package-manager record.

Remove:
- `~/.ctx/config.json`
- `~/.ctx/data/`
- `~/.ctx/logs/`
- `~/.ctx/tmp/`
- managed model/cache files under the app-owned directory, if used

Do not remove automatically:
- Homebrew installation metadata
- binaries placed in PATH by package managers
- provider-managed global caches outside the app-owned directory unless explicitly opted in later

Suggested behavior:
- interactive confirmation by default
- `--force` for non-interactive removal
- `--json` for machine-readable output
- clear summary of what will be deleted before confirmation

## Automatic Startup After Restart

For the current local-first design, there should be no required always-on service in V1.

That is the simplest and most reliable answer.

Why:
- LanceDB is embedded, not a server
- FastEmbed runs in-process, not as a daemon
- the CLI only needs local files and model assets

Result:
- after laptop restart, the tool still works immediately when the user runs `ctx ...`
- there is nothing the user needs to restart manually

### Optional background mode later

If we later add features like:
- background file watching
- automatic knowledge ingestion
- prewarming models
- local chat daemon

then add an optional companion service:
- `ctxd`

Install targets:
- `launchd` LaunchAgent on macOS
- `systemd --user` service on Linux
- Task Scheduler or Windows service wrapper on Windows

This should be optional, not required for core operation.

## Technical Plan

### Runtime

- `bun`
- `bunli`
- TypeScript
- `zod`

### Core libraries

- `lancedb`
- `fastembed`
- local file/markdown loaders
- `dotenv` only if needed for optional providers

### Internal modules

- `src/commands/`
- `src/config/`
- `src/storage/`
- `src/memory/`
- `src/embeddings/`
- `src/retrieval/`
- `src/chat/`
- `src/output/`

## Key Flows

### Setup flow

1. Resolve paths
2. Create folders
3. Initialize LanceDB
4. Initialize embedding provider
5. Save config
6. Run smoke checks

### Uninstall flow

1. Resolve managed paths
2. Show deletion summary
3. Confirm unless `--force` is set
4. Delete managed files and folders
5. Print completion summary

### Ingestion flow

1. Read text, file, or stdin
2. Normalize content
3. Chunk content
4. Generate embeddings locally
5. Store chunks and metadata in LanceDB

### Search flow

1. Embed query locally
2. Run vector and keyword retrieval
3. Merge and rank results
4. Render concise output

### Chat flow

1. Load recent session history
2. Retrieve relevant memory
3. Build context window
4. If a local chat provider is configured, call it
5. Persist the response and session state

## Delivery Phases

### Phase 1: Foundation

- Scaffold Bunli app
- Add `setup`, `uninstall`, `doctor`, and config handling
- Implement local directory and LanceDB initialization

### Phase 2: Local Embeddings

- Add `FastEmbed` provider abstraction
- Download/warm default model during setup
- Add embedding smoke tests

### Phase 3: Memory Core

- Implement `memory add`
- Implement `memory search`
- Add chunking and metadata handling

### Phase 4: Session Memory

- Implement `chat`
- Persist session history and message timeline
- Implement `sessions list` and `sessions show`

### Phase 5: Packaging

- Produce standalone binaries
- Add GitHub Releases workflow
- Add checksum generation
- Add Homebrew tap/formula
- Add direct install script
- Add completion install
- Add platform install docs

### Phase 6: Optional Daemon

- Add `ctxd`
- Add auto-start installers for supported OSes
- Add background sync/watch features only if needed

## Risks To Avoid

- Reintroducing external service dependencies too early
- Making a daemon mandatory before there is a real need
- Coupling storage, embeddings, and chat provider logic
- Hiding model download behavior from the user
- Letting first-run setup fail without clear recovery steps
- Making `ctx uninstall` remove anything outside the app-owned footprint

## Current Recommendation

Build V1 as:

- single-user
- local-first
- offline-capable after initial setup
- no required background process

## Open Questions

1. Should V1 include local chat generation, or should it first ship as memory ingest and retrieval only?
2. Should the default setup install shell completions automatically or ask first?
3. Should downloaded embedding model assets live under `~/.ctx/models/` explicitly, or should we rely on the provider’s default cache path?
