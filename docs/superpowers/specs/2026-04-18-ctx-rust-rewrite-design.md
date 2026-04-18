# ctx Rust Rewrite Design

Date: 2026-04-18
Status: Proposed

## Summary

Rewrite `ctx` from Bun/Bunli + TypeScript to a single Rust binary built with `clap`.

The Rust release targets full command parity for:

- `setup`
- `uninstall`
- `doctor`
- `update`
- `config show`
- `memory add`
- `memory search`

Breaking changes are acceptable when they produce a cleaner Rust-native design. The main deliberate break is data ownership: user memory must live in OS-standard app-data locations so updates, reinstalls, and normal uninstall flows do not destroy stored memory.

## Goals

- Replace the Bun/Bunli CLI and runtime with a Rust-native toolchain.
- Preserve the product scope of the current CLI in the first Rust release.
- Store durable user data in OS-standard app-data directories.
- Keep model downloads and other disposable artifacts in OS-standard cache directories.
- Use LanceDB from Rust as the primary retrieval store.
- Keep uninstall non-destructive by default.
- Provide an explicit import path from the current TypeScript store.

## Non-Goals

- Preserve byte-for-byte compatibility with the current JSON output.
- Preserve the current `~/.ctx` on-disk layout.
- Support multiple embedding providers in the first release.
- Build a distributed or remote service.
- Optimize for very large corpora before the local single-user design is stable.

## Architecture

The Rust binary should have four layers:

1. CLI
2. Application services
3. Storage and retrieval
4. System integration

### CLI

`clap` owns parsing, help output, defaults, validation, subcommand nesting, and prompts.

### Application Services

Application services implement command behavior for setup, uninstall, doctor, update, config, ingest, and search. Command handlers should stay thin and delegate immediately to services.

### Storage And Retrieval

LanceDB is the primary store for retrieval data. It holds documents, chunks, vectors, and search indexes. The Rust rewrite should use LanceDB as a real search engine, not just a persistence layer with in-memory ranking bolted on top.

### System Integration

System integration owns path resolution, config discovery, model cache management, release download logic, and destructive confirmations.

## Path Policy

The rewrite separates durable data, lightweight config, and disposable cache.

### macOS

- Data: `~/Library/Application Support/ctx/`
- Cache: `~/Library/Caches/ctx/`

### Linux

- Data: `${XDG_DATA_HOME:-~/.local/share}/ctx/`
- Cache: `${XDG_CACHE_HOME:-~/.cache}/ctx/`

### Windows

- Data: `%AppData%/ctx/`
- Cache: `%LocalAppData%/ctx/cache/`

### Data Directory Layout

- `config.toml` or `config.json`
- `db/`
- `logs/`
- `profiles/` if profile support remains justified after implementation review

### Cache Directory Layout

- `models/`
- `tmp/`

## Command Behavior

### `setup`

- Create required data and cache directories.
- Write default config when missing.
- Initialize LanceDB tables and indexes.
- Optionally warm the embedding runtime.

### `memory add`

- Accept exactly one input source: `--file`, `--text`, or `--stdin`.
- Normalize input.
- Chunk content with configured defaults or explicit flags.
- Generate embeddings in batches.
- Persist document and chunk rows into LanceDB in app-data storage.

### `memory search`

- Embed the query.
- Run vector search and full-text search.
- Merge and rerank results in Rust.
- Return top results in text or JSON mode.

### `config show`

- Print effective config, including resolved paths, retrieval defaults, and embedding settings.

### `doctor`

- Verify data and cache paths are writable.
- Verify embedding runtime health.
- Verify LanceDB tables and indexes are ready.

### `update`

- Replace the installed binary only.
- Never touch app-data memory or user config.

### `uninstall`

- Remove app-managed binary integrations.
- Preserve user memory by default.
- Optionally clear disposable cache.

### Destructive Delete

Data deletion must be explicit. Use either:

- `purge`
- `uninstall --purge-data`

This flow must require clear confirmation and must state exactly which directories will be deleted.

## Storage Design

Use two primary LanceDB tables.

### `documents`

One row per source item:

- `id`
- `source_type`
- `source_path`
- `source_hash`
- `title`
- `tags`
- `created_at`
- `updated_at`
- structured metadata

### `chunks`

One row per chunk:

- `id`
- `document_id`
- `chunk_index`
- `content`
- `content_hash`
- `token_estimate`
- `title`
- `source_path`
- `tags`
- `created_at`
- structured metadata
- embedding vector

The Rust version should avoid duplicate vector representations unless LanceDB requires them operationally. The current TypeScript code stores both native vectors and JSON-encoded vectors, then sometimes computes ranking after loading all chunks into memory. The rewrite should reduce that duplication and lean on LanceDB-native indexing and query capabilities.

## Ingest Flow

1. Validate that exactly one input source is present.
2. Read and normalize content.
3. Chunk content using configured `chunk_size` and `chunk_overlap`.
4. Generate embeddings in batches.
5. Write the document row and chunk rows.
6. Maintain search indexes in the write path.

## Search Flow

1. Embed the query.
2. Run vector search in LanceDB against chunk vectors.
3. Run full-text search in LanceDB against chunk content.
4. Merge results by chunk id.
5. Apply explicit hybrid reranking in Rust.
6. Return top `k` results.

The default reranking formula should preserve the current product shape while making it explicit and configurable:

`final_score = vector_weight * semantic_score + keyword_weight * lexical_score`

Initial default weights should stay close to the current behavior:

- `vector_weight = 0.7`
- `keyword_weight = 0.3`

## Embedding Runtime

The first Rust release should support one local embedding provider with CPU-first defaults. Keep the provider behind a trait or equivalent abstraction so a second provider can be added later without rewriting command logic.

Model files belong in the OS cache directory, not the data directory.

## Config Design

Config should be typed and versioned. It should include:

- resolved data and cache paths
- default `top_k`
- default `chunk_size`
- default `chunk_overlap`
- default output mode
- embedding provider and model
- hybrid ranking weights

`config show` should expose the effective values after path resolution and default expansion.

## Output Contract

Support both text and JSON output.

Text output should remain optimized for terminal use:

- clear summary fields
- ranked search results
- truncated previews where needed

JSON output should be stable within the Rust release line, but does not need to match the TypeScript output byte-for-byte.

## Testing Strategy

### Unit Tests

- chunking
- path resolution
- tag parsing
- config loading and validation
- reranking logic

### Integration Tests

- `setup`
- `memory add`
- `memory search`
- `config show`
- `doctor`

Use temporary app-data and cache roots so tests do not touch real user state.

### CLI Snapshot Tests

- text mode output
- JSON mode output
- error messages for invalid input combinations

### Release Smoke Tests

- install
- update
- uninstall
- persistence of memory across update and reinstall
- destructive purge path

## Migration Strategy

Migration should be explicit and reversible.

The Rust binary should either:

- provide `ctx import-ts`, or
- provide a one-time import command with equivalent behavior

The importer should:

- read the current TypeScript config and data
- rewrite data into the new app-data layout
- leave the old store untouched
- never delete legacy data automatically

## Key Design Decisions

- Full parity is the first-release scope.
- Breaking changes are acceptable when they improve the Rust-native design.
- LanceDB remains the retrieval store, but the rewrite should use it more natively.
- User memory moves from install-owned paths to OS-standard app-data locations.
- Cacheable artifacts move to OS-standard cache locations.
- Uninstall preserves memory by default.
- Destructive deletion becomes an explicit purge operation.
- Migration from the TypeScript version is explicit, not automatic and destructive.

## Risks And Mitigations

### Embedding Runtime Complexity

Risk: local inference and model management can dominate the rewrite.

Mitigation: support one provider first, keep the abstraction narrow, and store models in cache.

### Search Behavior Drift

Risk: retrieval quality changes when moving from the current hybrid implementation to a more native LanceDB flow.

Mitigation: keep the hybrid weighting explicit, test ranking behavior, and compare sample queries during migration.

### User Confusion Around Uninstall

Risk: users may expect uninstall to delete all data because that is common in developer tools.

Mitigation: document the new behavior clearly and reserve deletion for an explicit purge command.

### Migration Friction

Risk: users may not move existing memory into the new store.

Mitigation: provide a dedicated import command and never delete the old store automatically.

## Open Questions

- Whether `profiles/` remains a first-release feature or is deferred until the rest of the Rust architecture is stable.
- Whether config should use TOML or JSON. TOML is a better fit for human-edited config, but JSON is closer to the current format.
