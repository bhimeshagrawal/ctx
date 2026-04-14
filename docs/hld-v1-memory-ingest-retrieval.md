# CTX V1 Memory Ingest/Retrieval High-Level Design

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build V1 of `ctx` as a single-user, local-first CLI that can ingest content into memory and retrieve it later with local embeddings and embedded storage.

**Architecture:** `ctx` is a Bun/Bunli CLI with no required background service. It stores all memory locally in LanceDB, generates embeddings locally with FastEmbed, and exposes a small command surface centered on setup, ingestion, retrieval, and diagnostics.

**Tech Stack:** Bun, Bunli, TypeScript, Zod, LanceDB, FastEmbed

---

## 1. Scope

### In scope

- `ctx setup`
- `ctx uninstall`
- `ctx doctor`
- `ctx memory add`
- `ctx memory search`
- `ctx config show`
- local config management
- local folder bootstrap
- local embeddings
- embedded storage
- keyword + vector retrieval

### Out of scope

- chat generation
- background daemon
- file watching
- multi-user support
- cloud sync
- hosted model providers

## 2. Product Goals

- Require no paid API
- Require no external database
- Work after laptop restart without manual service recovery
- Keep setup to one explicit command
- Be useful offline after initial install/model download

## 3. System Context

The system has one executable, one local app directory, one embedded database, and one local embedding runtime.

### Runtime components

- CLI process: command parsing, orchestration, output
- LanceDB: embedded storage under `~/.ctx/data/`
- FastEmbed: in-process embedding generation
- local filesystem: config, logs, temp files, source metadata

## 4. Top-Level Architecture

### Layers

- CLI layer
  - Bunli command definitions
- Application layer
  - setup, ingest, search, diagnostics services
- Data layer
  - LanceDB repositories
- Embedding layer
  - FastEmbed provider abstraction
- Output layer
  - terminal tables, summaries, error rendering

### Design constraints

- No command should require an always-on background process
- Setup must be idempotent
- Retrieval must work even if only local disk is available
- Embedding provider must be swappable behind an interface

## 5. User Flows

### Flow A: First-time setup

1. User installs the `ctx` binary
2. User runs `ctx setup`
3. CLI creates `~/.ctx/` directories
4. CLI initializes config and LanceDB tables
5. CLI initializes FastEmbed and verifies a sample embedding
6. CLI prints usable next steps

### Flow D: Uninstall local state

1. User runs `ctx uninstall`
2. CLI shows which local paths it manages
3. CLI asks for confirmation unless `--force` is used
4. CLI removes managed local state
5. CLI prints a completion summary and notes that package-manager uninstall is separate

### Flow B: Add memory

1. User runs `ctx memory add --file note.md`
2. CLI reads content
3. CLI normalizes and chunks content
4. CLI generates embeddings locally
5. CLI writes chunks and metadata into LanceDB
6. CLI prints ingest summary

### Flow C: Search memory

1. User runs `ctx memory search "query"`
2. CLI generates a local query embedding
3. CLI performs vector search plus keyword search
4. CLI merges and ranks results
5. CLI prints concise matches with source metadata

## 6. Data Model

### Table: `documents`

Purpose:
- one row per source object submitted by the user

Representative fields:
- `id`
- `source_type`
- `source_path`
- `source_hash`
- `title`
- `created_at`
- `updated_at`
- `tags`
- `metadata`

### Table: `chunks`

Purpose:
- one row per searchable content chunk

Representative fields:
- `id`
- `document_id`
- `chunk_index`
- `content`
- `content_hash`
- `embedding`
- `token_estimate`
- `created_at`
- `metadata`

### Table: `profiles`

Purpose:
- store procedural preferences for the local user

Representative fields:
- `id`
- `name`
- `default_top_k`
- `default_chunk_size`
- `default_chunk_overlap`
- `output_mode`
- `embedding_model`
- `metadata`

## 7. Retrieval Strategy

V1 retrieval uses a hybrid approach:

- vector similarity from local embeddings
- keyword matching from normalized content
- simple weighted rank merge

This is enough for V1 and avoids overbuilding advanced ranking logic.

## 8. Installation and Restart Behavior

### Installation

- distribute standalone binaries via GitHub Releases
- use Homebrew as the primary install path for macOS/Linux
- provide a direct install script as a convenience layer
- `ctx setup` performs one-time initialization

### Uninstall behavior

- `ctx uninstall` removes app-managed local state only
- package-manager removal remains the responsibility of Homebrew or the user
- the binary itself is not self-deleted in V1

### Distribution channels

- GitHub Releases
  - canonical source of release artifacts
- Homebrew
  - primary install path for macOS/Linux
- direct install script
  - convenience path for users who do not want to set up Homebrew
- Windows binaries via GitHub Releases

### Packaging constraints

- release one binary per supported platform/architecture
- publish checksums with each release
- keep install instructions aligned with binary naming
- do not make npm the primary distribution channel for V1

### Restart behavior

- no daemon required
- nothing needs to auto-start on reboot
- the CLI remains usable because LanceDB and FastEmbed are local/in-process

### Future optional daemon

Only add `ctxd` if background indexing or prewarming becomes necessary.

## 9. Verification Strategy

- setup smoke test
- ingest smoke test
- retrieval smoke test
- file path permission checks
- corrupted config handling

## 10. Implementation Sequence

- [ ] Create project skeleton and command registry
- [ ] Implement config/path resolution
- [ ] Implement `ctx setup`
- [ ] Implement LanceDB initialization
- [ ] Implement FastEmbed provider
- [ ] Implement ingestion pipeline
- [ ] Implement search pipeline
- [ ] Implement `ctx doctor`
- [ ] Implement `ctx config show`
- [ ] Add tests and packaging

## 11. Acceptance Criteria

- `ctx setup` succeeds on a clean machine with writable home directory
- `ctx uninstall --force` removes app-managed local state without touching package-managed binaries
- `ctx memory add` ingests a local file and stores searchable chunks
- `ctx memory search` returns ranked results from local storage
- the tool works after a laptop restart without extra setup
- no hosted API is required
