# ctx Rust Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Bun/Bunli TypeScript CLI with a Rust `clap` binary that preserves the current command set, uses OS-standard app-data and cache locations, keeps user memory durable across update and reinstall, and uses LanceDB as the retrieval store.

**Architecture:** Build a single Rust crate with thin `clap` command parsing, service modules for command behavior, a typed config and path layer, LanceDB-backed storage for documents and chunks, and a provider abstraction for embeddings. Replace Bun packaging with Cargo-based builds, keep install and update behavior, and make uninstall non-destructive by default.

**Tech Stack:** Rust, Cargo, `clap`, `serde`, `toml`, `directories`, `tokio`, LanceDB Rust SDK, Rust test harness, GitHub Actions.

---

### Task 1: Replace project scaffolding with a Rust crate

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Create: `src/lib.rs`
- Modify: `README.md`
- Modify: `.github/workflows/release.yml`
- Remove or supersede: `package.json`, `bun.lock`, `bunli.config.ts`, `tsconfig.json`

- [ ] **Step 1: Write failing CLI smoke tests**

Create `tests/cli_smoke.rs` with checks for top-level help and subcommand wiring.

```rust
use std::process::Command;

#[test]
fn help_lists_core_commands() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--help")
        .output()
        .expect("run ctx --help");

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("setup"));
    assert!(text.contains("uninstall"));
    assert!(text.contains("doctor"));
    assert!(text.contains("update"));
    assert!(text.contains("config"));
    assert!(text.contains("memory"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test help_lists_core_commands --test cli_smoke`
Expected: FAIL because the Rust crate does not exist yet.

- [ ] **Step 3: Create the Cargo crate and minimal clap entrypoint**

Add a binary crate that exposes the command tree:

```rust
#[derive(clap::Parser)]
#[command(name = "ctx")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    Setup(SetupArgs),
    Uninstall(UninstallArgs),
    Doctor(DoctorArgs),
    Update(UpdateArgs),
    Config(ConfigCommand),
    Memory(MemoryCommand),
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test help_lists_core_commands --test cli_smoke`
Expected: PASS and help text includes the command names.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/main.rs src/lib.rs src/cli.rs tests/cli_smoke.rs README.md .github/workflows/release.yml
git commit -m "🎉 chore: scaffold Rust ctx CLI"
```

### Task 2: Port paths, config, and non-destructive lifecycle semantics

**Files:**
- Create: `src/paths.rs`
- Create: `src/config.rs`
- Create: `src/commands/setup.rs`
- Create: `src/commands/uninstall.rs`
- Create: `src/commands/doctor.rs`
- Create: `src/commands/config.rs`
- Test: `tests/paths_config.rs`

- [ ] **Step 1: Write failing path and config tests**

Create tests for OS-aware path resolution, config persistence, and non-destructive uninstall defaults.

```rust
#[test]
fn resolved_paths_use_separate_data_and_cache_roots() {
    let paths = CtxPaths::from_roots("/tmp/ctx-data", "/tmp/ctx-cache");
    assert_eq!(paths.db_dir, "/tmp/ctx-data/db");
    assert_eq!(paths.models_dir, "/tmp/ctx-cache/models");
}

#[test]
fn uninstall_keeps_data_without_purge_flag() {
    let plan = uninstall_plan(false);
    assert!(!plan.delete_data_dir);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test paths_config`
Expected: FAIL because `CtxPaths` and uninstall planning do not exist.

- [ ] **Step 3: Implement typed paths and config**

Use `directories` to resolve standard app-data and cache paths, and persist a versioned config file:

```rust
pub struct CtxPaths {
    pub data_root: PathBuf,
    pub cache_root: PathBuf,
    pub db_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub models_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub config_path: PathBuf,
}
```

```rust
#[derive(Serialize, Deserialize)]
pub struct CtxConfig {
    pub version: u32,
    pub defaults: Defaults,
    pub embeddings: EmbeddingsConfig,
    pub ranking: RankingConfig,
}
```

- [ ] **Step 4: Implement lifecycle commands**

`setup` creates directories and writes config. `config show` prints the effective config. `doctor` validates writable paths and runtime readiness. `uninstall` removes install-managed artifacts and cache, but preserves data unless `--purge-data` is passed.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --test paths_config`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/paths.rs src/config.rs src/commands/setup.rs src/commands/uninstall.rs src/commands/doctor.rs src/commands/config.rs tests/paths_config.rs
git commit -m "✨ feat: add Rust path and lifecycle services"
```

### Task 3: Port ingest pipeline and embedding provider abstraction

**Files:**
- Create: `src/input.rs`
- Create: `src/chunking.rs`
- Create: `src/normalize.rs`
- Create: `src/embeddings/mod.rs`
- Create: `src/embeddings/provider.rs`
- Create: `src/embeddings/local.rs`
- Create: `src/commands/memory/add.rs`
- Test: `tests/ingest.rs`

- [ ] **Step 1: Write failing ingest tests**

Cover mutually exclusive inputs, chunk overlap validation, and text ingest metadata.

```rust
#[test]
fn read_input_requires_exactly_one_source() {
    let result = read_input(None, None, false);
    assert!(result.is_err());
}

#[test]
fn chunker_rejects_overlap_equal_to_size() {
    let result = chunk_text("hello", 10, 10);
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test ingest`
Expected: FAIL because the ingest helpers do not exist.

- [ ] **Step 3: Implement input, normalization, and chunking**

Port the current input contract and chunking semantics:

```rust
pub enum InputSource {
    File(PathBuf),
    Text(String),
    Stdin,
}

pub struct InputPayload {
    pub source_type: SourceType,
    pub source_path: Option<PathBuf>,
    pub title: Option<String>,
    pub content: String,
}
```

- [ ] **Step 4: Implement the embedding abstraction**

Start with one local CPU-first provider behind a trait:

```rust
#[async_trait::async_trait]
pub trait EmbeddingProvider {
    async fn init(&self) -> anyhow::Result<()>;
    async fn dimension(&self) -> anyhow::Result<usize>;
    async fn embed(&self, texts: &[String]) -> anyhow::Result<Vec<Vec<f32>>>;
    async fn embed_query(&self, query: &str) -> anyhow::Result<Vec<f32>>;
    async fn health_check(&self) -> anyhow::Result<String>;
}
```

- [ ] **Step 5: Wire `memory add`**

Create the Rust `memory add` handler to read input, normalize, chunk, embed, and call the storage layer.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --test ingest`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/input.rs src/chunking.rs src/normalize.rs src/embeddings src/commands/memory/add.rs tests/ingest.rs
git commit -m "✨ feat: port ingest pipeline to Rust"
```

### Task 4: Implement LanceDB-backed storage and hybrid retrieval

**Files:**
- Create: `src/storage/mod.rs`
- Create: `src/storage/schema.rs`
- Create: `src/storage/db.rs`
- Create: `src/storage/documents.rs`
- Create: `src/storage/chunks.rs`
- Create: `src/search.rs`
- Create: `src/ranking.rs`
- Create: `src/commands/memory/search.rs`
- Test: `tests/search.rs`

- [ ] **Step 1: Write failing ranking and search tests**

Mirror the current ranking behavior and search shape.

```rust
#[test]
fn hybrid_ranking_merges_vector_and_keyword_scores() {
    let ranked = rank_results(vec![candidate("a", 0.8, 1.0)], 5, 0.7, 0.3);
    assert_eq!(ranked.len(), 1);
    assert!((ranked[0].final_score - 0.86).abs() < 0.001);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test search`
Expected: FAIL because ranking and LanceDB integration do not exist.

- [ ] **Step 3: Implement LanceDB schema and repositories**

Define `documents` and `chunks` tables with structured metadata and vectors, then add repository methods for insert and search.

- [ ] **Step 4: Implement hybrid retrieval**

Run vector search and full-text search through LanceDB, merge by chunk id, then rerank in Rust:

```rust
final_score = vector_weight * semantic_score + keyword_weight * lexical_score;
```

- [ ] **Step 5: Wire `memory search`**

Expose `--top-k`, `--tag`, and `--json`, and return text or JSON output through a shared renderer.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --test search`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/storage src/search.rs src/ranking.rs src/commands/memory/search.rs tests/search.rs
git commit -m "✨ feat: add LanceDB-backed memory search"
```

### Task 5: Port update logic, install flow, docs, and release packaging

**Files:**
- Create: `src/update.rs`
- Create: `src/commands/update.rs`
- Modify: `install.sh`
- Modify: `.github/workflows/release.yml`
- Modify: `README.md`
- Test: `tests/release.rs`

- [ ] **Step 1: Write failing release tests**

Mirror the current asset naming and release URL expectations.

```rust
#[test]
fn release_base_url_uses_latest_by_default() {
    assert_eq!(
        release_base_url("owner/repo", None),
        "https://github.com/owner/repo/releases/latest/download"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test release`
Expected: FAIL because update helpers do not exist.

- [ ] **Step 3: Implement update helpers and command**

Port asset naming, release URL construction, and binary replacement logic for the Rust binary layout.

- [ ] **Step 4: Update install and release automation**

Replace Bun build steps with Cargo build steps and adjust install assets for the Rust binary output.

- [ ] **Step 5: Update docs**

Rewrite `README.md` so local development, setup, install, update, uninstall, and data persistence all reflect the Rust implementation and the new storage semantics.

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --test release`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add src/update.rs src/commands/update.rs install.sh .github/workflows/release.yml README.md tests/release.rs
git commit -m "🚀 ci: switch release flow to Cargo"
```

### Task 6: Remove Bun entrypoints, run full verification, and ship on main

**Files:**
- Remove or archive: `src/**/*.ts`
- Remove or archive: `package.json`, `bun.lock`, `bunli.config.ts`, `tsconfig.json`
- Modify: repository root build metadata as needed

- [ ] **Step 1: Remove superseded TypeScript runtime files**

Delete the old Bun CLI entrypoint, command modules, and TypeScript-only build metadata once the Rust replacement passes tests.

- [ ] **Step 2: Run the full test suite**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 3: Run targeted CLI smoke checks**

Run:

```bash
cargo run -- --help
cargo run -- config show --json
cargo run -- memory add --text "ctx smoke test" --tag ctx,test --json
cargo run -- memory search "smoke test" --json
```

Expected: all commands succeed and read or write only the test-configured paths.

- [ ] **Step 4: Review git diff and commit**

```bash
git add Cargo.toml src tests install.sh README.md .github/workflows/release.yml
git rm -r src/commands src/config src/doctor src/embeddings src/ingest src/output src/retrieval src/setup src/storage src/types src/update package.json bun.lock bunli.config.ts tsconfig.json
git commit -m "💥 feat: rewrite ctx from Bun to Rust"
```

- [ ] **Step 5: Push directly to main**

```bash
git push origin main
```

Expected: remote `main` updates with the Rust rewrite after local verification passes.
