# CTX V1 Memory Ingest/Retrieval Low-Level Design

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Define the concrete modules, files, data structures, command contracts, and execution order for implementing V1 memory ingest and retrieval.

**Architecture:** A Bunli CLI orchestrates config loading, local directory bootstrap, LanceDB access, FastEmbed embeddings, chunking, and ranked search. The system is intentionally single-process and single-user, with all persistent state stored under `~/.ctx/`.

**Tech Stack:** Bun, Bunli, TypeScript, Zod, LanceDB, FastEmbed

---

## 1. Repository Layout

Create this structure:

```text
src/
  index.ts
  commands/
    setup.ts
    uninstall.ts
    doctor.ts
    config-show.ts
    memory-add.ts
    memory-search.ts
  config/
    paths.ts
    schema.ts
    load-config.ts
    save-config.ts
  storage/
    lancedb.ts
    tables.ts
    repositories/
      documents-repo.ts
      chunks-repo.ts
      profiles-repo.ts
  embeddings/
    provider.ts
    fastembed-provider.ts
  ingest/
    read-input.ts
    normalize.ts
    chunk.ts
    ingest-service.ts
  retrieval/
    keyword-search.ts
    vector-search.ts
    rank.ts
    search-service.ts
  doctor/
    doctor-service.ts
  output/
    render-text.ts
    render-json.ts
  types/
    document.ts
    chunk.ts
    profile.ts
```

## 2. App Directory Contract

Use resolved paths from one place only.

### Path resolver

File:
- `src/config/paths.ts`

Responsibilities:
- detect home directory
- resolve `~/.ctx`
- expose:
  - `rootDir`
  - `dataDir`
  - `logsDir`
  - `tmpDir`
  - `configPath`

Do not scatter path logic across commands.

## 3. Config Contract

File:
- `src/config/schema.ts`

Suggested config shape:

```ts
type CtxConfig = {
  version: 1;
  paths: {
    rootDir: string;
    dataDir: string;
    logsDir: string;
    tmpDir: string;
  };
  defaults: {
    topK: number;
    chunkSize: number;
    chunkOverlap: number;
    outputMode: "text" | "json";
  };
  embeddings: {
    provider: "fastembed";
    model: string;
  };
};
```

Validation:
- use `zod`
- reject partial or malformed configs
- on invalid config, `ctx doctor` should explain recovery steps

## 4. LanceDB Table Definitions

File:
- `src/storage/tables.ts`

### `documents`

```ts
type DocumentRow = {
  id: string;
  sourceType: "file" | "stdin" | "text";
  sourcePath: string | null;
  sourceHash: string;
  title: string | null;
  tags: string[];
  createdAt: string;
  updatedAt: string;
  metadata: string;
};
```

### `chunks`

```ts
type ChunkRow = {
  id: string;
  documentId: string;
  chunkIndex: number;
  content: string;
  contentHash: string;
  tokenEstimate: number;
  embedding: number[];
  createdAt: string;
  metadata: string;
};
```

### `profiles`

```ts
type ProfileRow = {
  id: string;
  name: string;
  defaultTopK: number;
  defaultChunkSize: number;
  defaultChunkOverlap: number;
  outputMode: "text" | "json";
  embeddingModel: string;
  metadata: string;
};
```

## 5. Command Contracts

### `ctx setup`

File:
- `src/commands/setup.ts`

Behavior:
- create app directories
- initialize config if missing
- initialize LanceDB connection and tables
- initialize FastEmbed provider
- run smoke checks

Exit criteria:
- valid config exists
- tables exist
- one sample embedding succeeds

### `ctx doctor`

File:
- `src/commands/doctor.ts`

Checks:
- config readable
- directories writable
- LanceDB reachable
- tables present
- embedding provider healthy

### `ctx uninstall`

File:
- `src/commands/uninstall.ts`

Flags:
- `--force`
- `--json`

Behavior:
- resolve managed app paths
- show deletion summary
- ask for confirmation unless `--force` is set
- remove app-managed files and folders
- print what was removed

Rules:
- only remove paths under the resolved app root
- do not remove the installed binary
- do not invoke Homebrew or other package managers
- missing paths should not be treated as fatal

### `ctx config show`

File:
- `src/commands/config-show.ts`

Behavior:
- print effective config
- support `--json`

### `ctx memory add`

File:
- `src/commands/memory-add.ts`

Flags:
- `--file <path>`
- `--text <value>`
- `--stdin`
- `--title <title>`
- `--tag <tag>` repeatable
- `--chunk-size <n>`
- `--chunk-overlap <n>`
- `--json`

Validation rules:
- exactly one input source required
- file path must exist for `--file`
- chunk overlap must be less than chunk size

### `ctx memory search <query>`

File:
- `src/commands/memory-search.ts`

Flags:
- `--top-k <n>`
- `--tag <tag>` repeatable
- `--json`

Behavior:
- embed query
- run vector search
- run keyword filter/search
- merge and rank
- render results

## 6. Embedding Provider Interface

File:
- `src/embeddings/provider.ts`

```ts
export interface EmbeddingProvider {
  readonly name: string;
  readonly model: string;
  init(): Promise<void>;
  embed(texts: string[]): Promise<number[][]>;
  healthCheck(): Promise<{ ok: boolean; detail: string }>;
}
```

Implementation:
- `src/embeddings/fastembed-provider.ts`

Rules:
- initialization must be explicit
- batch embeddings where possible
- surface model download/setup failures clearly

## 7. Ingestion Pipeline

### `read-input.ts`

Responsibilities:
- read from file
- read from stdin
- accept direct text input

### `normalize.ts`

Responsibilities:
- normalize line endings
- trim excessive blank lines
- preserve semantic content

### `chunk.ts`

Chunking strategy for V1:
- character-based chunking
- default chunk size: `1200`
- default overlap: `150`

Why:
- simpler than tokenizer-based chunking
- good enough for V1 retrieval

### ` ingest-service.ts`

Responsibilities:
- validate inputs
- compute source hash
- create document row
- split content into chunks
- generate embeddings
- write document and chunk rows

## 8. Retrieval Pipeline

### `vector-search.ts`

Responsibilities:
- embed query
- search chunks by vector similarity

### `keyword-search.ts`

Responsibilities:
- perform simple lexical matching over `content`
- optionally prefilter by tags or source metadata

### `rank.ts`

V1 rank formula:
- normalize vector score
- normalize keyword hit score
- compute weighted score, default:
  - vector: `0.7`
  - keyword: `0.3`

### `search-service.ts`

Responsibilities:
- orchestrate vector + keyword search
- merge duplicate chunk hits
- attach document metadata
- return top `k` ranked results

## 9. Output Contract

### Text mode

Show:
- rank
- content preview
- source title or path
- tags

### JSON mode

Return stable fields:
- `documentId`
- `chunkId`
- `score`
- `content`
- `title`
- `sourcePath`
- `tags`

## 10. Error Handling

Handle explicitly:
- missing home directory
- unwritable app directory
- invalid config
- missing input source
- empty content after normalization
- embedding initialization failure
- LanceDB open failure
- search on empty database
- uninstall against a partially deleted app directory
- uninstall refused because a target path resolves outside the app root

Errors should be short in text mode and structured in JSON mode.

## 11. Test Plan

### Unit tests

- path resolution
- config validation
- chunking
- normalization
- rank merge logic

### Integration tests

- `ctx setup` creates directories and config
- `ctx uninstall --force` removes managed directories and config
- `ctx memory add --text ...` stores document and chunks
- `ctx memory search ...` returns expected seeded content
- `ctx doctor` reports healthy state after setup

### Smoke tests

- install binary
- run setup
- ingest sample markdown
- retrieve sample term

## 12. Execution Plan

- [ ] Task 1: Scaffold Bunli app and register commands
  - Files: `src/index.ts`, `src/commands/*`
  - Verify: `ctx --help`

- [ ] Task 2: Implement config/path layer
  - Files: `src/config/*`
  - Verify: unit tests for path and schema

- [ ] Task 3: Implement LanceDB bootstrap and repositories
  - Files: `src/storage/*`
  - Verify: integration test for table initialization

- [ ] Task 4: Implement FastEmbed provider abstraction
  - Files: `src/embeddings/*`
  - Verify: one real embedding generated in test or smoke flow

- [ ] Task 5: Implement ingestion pipeline
  - Files: `src/ingest/*`, `src/commands/memory-add.ts`
  - Verify: document and chunk rows created

- [ ] Task 6: Implement retrieval pipeline
  - Files: `src/retrieval/*`, `src/commands/memory-search.ts`
  - Verify: seeded document is returned for relevant query

- [ ] Task 7: Implement diagnostics and config display
  - Files: `src/doctor/*`, `src/commands/doctor.ts`, `src/commands/config-show.ts`, `src/commands/uninstall.ts`
  - Verify: healthy status after setup

- [ ] Task 8: Add output modes, tests, and package build
  - Files: `src/output/*`, test files, build config
  - Verify: text and JSON output both stable

## 13. Done Definition

Implementation is complete when:

- setup is idempotent
- ingestion works from text, file, and stdin
- retrieval returns ranked local results
- diagnostics can identify broken setup
- no external DB or paid API is required
