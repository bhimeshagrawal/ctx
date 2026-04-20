# ctx MCP Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a public-stable MCP server to `ctx` with `stdio` and HTTP/SSE transports, full tool/resource/prompt support, and near-full CLI parity over shared Rust services.

**Architecture:** Extract typed application services from the current CLI command handlers, then build an `rmcp`-based server core over those services. Keep the CLI as a thin adapter over the same service layer, and expose `stdio` plus localhost-only HTTP/SSE transports through a new `ctx mcp serve` command family.

**Tech Stack:** Rust, `clap`, `tokio`, `serde`, `schemars`, `rmcp`, `reqwest`, LanceDB, FastEmbed

---

### Task 1: Add MCP dependencies and CLI surface

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/cli.rs`
- Modify: `src/commands/mod.rs`
- Create: `src/commands/mcp.rs`
- Test: `tests/cli_smoke.rs`

- [ ] **Step 1: Write the failing CLI test for the new `mcp` command**

```rust
use std::process::Command;

#[test]
fn help_lists_mcp_command() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .arg("--help")
        .output()
        .expect("run ctx --help");

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("mcp"));
}

#[test]
fn mcp_help_lists_serve_and_transport_flags() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--help"])
        .output()
        .expect("run ctx mcp serve --help");

    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("--transport"));
    assert!(text.contains("stdio"));
    assert!(text.contains("http"));
}
```

- [ ] **Step 2: Run the CLI test to verify it fails**

Run: `cargo test --test cli_smoke help_lists_mcp_command mcp_help_lists_serve_and_transport_flags`
Expected: FAIL because the `mcp` command does not exist yet.

- [ ] **Step 3: Add the MCP dependency set and CLI types**

```toml
[dependencies]
anyhow = "1.0"
arrow-array = "57.3"
arrow-schema = "57.3"
async-trait = "0.1"
chrono = { version = "0.4", features = ["clock", "serde"] }
clap = { version = "4.5", features = ["derive"] }
directories = "5.0"
fastembed = "5.13"
futures = "0.3"
lancedb = "0.27"
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }
rmcp = { version = "1.5.0", features = ["server", "transport-io", "transport-streamable-http-server", "schemars"] }
schemars = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sha2 = "0.10"
tokio = { version = "1.42", features = ["fs", "io-std", "io-util", "macros", "net", "rt-multi-thread"] }
toml = "0.8"
uuid = { version = "1.11", features = ["serde", "v4"] }
```

```rust
#[derive(Debug, Subcommand)]
pub enum Commands {
    Setup(SetupArgs),
    Uninstall(UninstallArgs),
    Doctor(DoctorArgs),
    Update(UpdateArgs),
    Config(ConfigArgs),
    Memory(MemoryArgs),
    Mcp(McpArgs),
}

#[derive(Debug, Args)]
pub struct McpArgs {
    #[command(subcommand)]
    pub command: McpCommand,
}

#[derive(Debug, Subcommand)]
pub enum McpCommand {
    Serve(McpServeArgs),
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum McpTransport {
    Stdio,
    Http,
}

#[derive(Debug, Args)]
pub struct McpServeArgs {
    #[arg(long, value_enum, default_value_t = McpTransport::Stdio)]
    pub transport: McpTransport,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, default_value_t = 0)]
    pub port: u16,
}
```

```rust
pub mod mcp;

pub async fn run(args: McpArgs) -> Result<()> {
    match args.command {
        McpCommand::Serve(args) => mcp::run(args).await,
    }
}
```

- [ ] **Step 4: Add the initial command adapter**

```rust
use anyhow::Result;

use crate::cli::McpServeArgs;

pub async fn run(_args: McpServeArgs) -> Result<()> {
    anyhow::bail!("mcp server is not implemented yet")
}
```

- [ ] **Step 5: Run the CLI test to verify it passes**

Run: `cargo test --test cli_smoke`
Expected: PASS with the `mcp` help output visible and the placeholder handler reachable.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/cli.rs src/commands/mod.rs src/commands/mcp.rs tests/cli_smoke.rs
git commit -m "feat: add mcp cli surface"
```

### Task 2: Extract shared runtime bootstrap and service contracts

**Files:**
- Create: `src/services/mod.rs`
- Create: `src/services/runtime.rs`
- Create: `src/services/types.rs`
- Modify: `src/lib.rs`
- Test: `tests/services_runtime.rs`

- [ ] **Step 1: Write the failing service bootstrap test**

```rust
use ctx::services::runtime::ServiceRuntime;
use tempfile::TempDir;

#[tokio::test]
async fn runtime_bootstrap_uses_explicit_roots() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();

    let runtime = ServiceRuntime::bootstrap(
        Some(data.path().to_path_buf()),
        Some(cache.path().to_path_buf()),
        false,
    )
    .await
    .unwrap();

    assert_eq!(runtime.paths.data_root, data.path());
    assert_eq!(runtime.paths.cache_root, cache.path());
}
```

- [ ] **Step 2: Run the runtime test to verify it fails**

Run: `cargo test --test services_runtime`
Expected: FAIL because `ctx::services` does not exist yet.

- [ ] **Step 3: Add the runtime and shared response types**

```rust
pub mod runtime;
pub mod types;
```

```rust
use anyhow::Result;

use crate::{
    config::{self, CtxConfig},
    embeddings::local::LocalEmbeddingProvider,
    paths::CtxPaths,
    storage::{self, CtxDatabase},
};

pub struct ServiceRuntime {
    pub paths: CtxPaths,
    pub config: CtxConfig,
    pub provider: LocalEmbeddingProvider,
    pub db: CtxDatabase,
}

impl ServiceRuntime {
    pub async fn bootstrap(
        data_root: Option<std::path::PathBuf>,
        cache_root: Option<std::path::PathBuf>,
        verbose_embeddings: bool,
    ) -> Result<Self> {
        let paths = CtxPaths::resolve(data_root, cache_root)?;
        let config = config::load_or_default(&paths).await?;
        let provider = LocalEmbeddingProvider::new(
            &config.embeddings.model,
            paths.models_dir.clone(),
            verbose_embeddings,
        );
        let db = storage::init_database(&paths, &provider).await?;

        Ok(Self {
            paths,
            config,
            provider,
            db,
        })
    }
}
```

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetupResponse {
    pub ok: bool,
    pub data_root: String,
    pub cache_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DoctorCheck {
    pub name: String,
    pub ok: bool,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DoctorResponse {
    pub ok: bool,
    pub checks: Vec<DoctorCheck>,
}
```

- [ ] **Step 4: Export the services module**

```rust
pub mod services;
```

- [ ] **Step 5: Run the runtime test to verify it passes**

Run: `cargo test --test services_runtime`
Expected: PASS with deterministic explicit-root bootstrap.

- [ ] **Step 6: Commit**

```bash
git add src/lib.rs src/services/mod.rs src/services/runtime.rs src/services/types.rs tests/services_runtime.rs
git commit -m "refactor: add shared service runtime"
```

### Task 3: Extract memory add and search into typed services

**Files:**
- Create: `src/services/memory.rs`
- Modify: `src/commands/memory/add.rs`
- Modify: `src/commands/memory/search.rs`
- Modify: `src/services/types.rs`
- Test: `tests/services_memory.rs`

- [ ] **Step 1: Write the failing service test for structured memory requests**

```rust
use ctx::services::{
    memory::{self, MemoryAddRequest, MemorySource, MemorySearchRequest},
    runtime::ServiceRuntime,
};
use tempfile::TempDir;

#[tokio::test]
async fn memory_service_adds_and_searches_text() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let runtime = ServiceRuntime::bootstrap(
        Some(data.path().to_path_buf()),
        Some(cache.path().to_path_buf()),
        false,
    )
    .await
    .unwrap();

    let add = memory::add(
        &runtime,
        MemoryAddRequest {
            source: MemorySource::Text {
                text: "ctx mcp design note".to_string(),
            },
            title: Some("design".to_string()),
            tags: vec!["ctx".to_string(), "mcp".to_string()],
            chunk_size: None,
            chunk_overlap: None,
        },
    )
    .await
    .unwrap();

    let search = memory::search(
        &runtime,
        MemorySearchRequest {
            query: "mcp design".to_string(),
            top_k: Some(5),
            tags: vec!["ctx".to_string()],
        },
    )
    .await
    .unwrap();

    assert!(add.ok);
    assert!(search.count >= 1);
}
```

- [ ] **Step 2: Run the memory service test to verify it fails**

Run: `cargo test --test services_memory`
Expected: FAIL because `src/services/memory.rs` does not exist yet.

- [ ] **Step 3: Define request and response types for memory**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MemorySource {
    Text { text: String },
    File { path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryAddRequest {
    pub source: MemorySource,
    pub title: Option<String>,
    pub tags: Vec<String>,
    pub chunk_size: Option<usize>,
    pub chunk_overlap: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySearchRequest {
    pub query: String,
    pub top_k: Option<usize>,
    pub tags: Vec<String>,
}
```

- [ ] **Step 4: Move the current memory logic into reusable services**

```rust
pub async fn add(runtime: &ServiceRuntime, request: MemoryAddRequest) -> Result<serde_json::Value> {
    let input = match request.source {
        MemorySource::Text { text } => crate::input::InputContent {
            content: text,
            title: None,
            source_path: None,
            source_type: crate::input::InputSourceType::Text,
        },
        MemorySource::File { path } => crate::input::read_input(Some(path), None, false).await?,
    };

    let normalized = crate::normalize::normalize_content(&input.content);
    let chunk_size = request
        .chunk_size
        .unwrap_or(runtime.config.defaults.chunk_size);
    let chunk_overlap = request
        .chunk_overlap
        .unwrap_or(runtime.config.defaults.chunk_overlap);
    let chunks = crate::chunking::chunk_text(&normalized, chunk_size, chunk_overlap)?;

    // Preserve the current add flow here and return a typed payload instead of rendering.
    Ok(serde_json::json!({
        "ok": true,
        "chunkCount": chunks.len()
    }))
}

pub async fn search(
    runtime: &ServiceRuntime,
    request: MemorySearchRequest,
) -> Result<crate::search::SearchResult> {
    crate::search::run_search(
        &runtime.db,
        &runtime.provider,
        &runtime.config,
        &request.query,
        request.top_k.unwrap_or(runtime.config.defaults.top_k),
        &request.tags,
    )
    .await
}
```

- [ ] **Step 5: Rewrite the CLI command handlers as adapters**

```rust
pub async fn run(args: MemoryAddArgs) -> Result<()> {
    let runtime = ServiceRuntime::bootstrap(None, None, !args.json).await?;
    let request = if let Some(text) = args.text {
        MemoryAddRequest {
            source: MemorySource::Text { text },
            title: args.title,
            tags: args.tags,
            chunk_size: args.chunk_size,
            chunk_overlap: args.chunk_overlap,
        }
    } else if let Some(path) = args.file {
        MemoryAddRequest {
            source: MemorySource::File { path },
            title: args.title,
            tags: args.tags,
            chunk_size: args.chunk_size,
            chunk_overlap: args.chunk_overlap,
        }
    } else {
        anyhow::bail!("stdin support should be bridged in a follow-up adapter helper");
    };

    let result = crate::services::memory::add(&runtime, request).await?;
    crate::output::render(&result, args.json)
}
```

- [ ] **Step 6: Run the service and existing ingest/search tests**

Run: `cargo test --test services_memory --test ingest --test search`
Expected: PASS with the CLI still using the extracted memory services.

- [ ] **Step 7: Commit**

```bash
git add src/services/memory.rs src/services/types.rs src/commands/memory/add.rs src/commands/memory/search.rs tests/services_memory.rs
git commit -m "refactor: extract memory services"
```

### Task 4: Extract setup, doctor, config, update, and uninstall into services

**Files:**
- Create: `src/services/system.rs`
- Modify: `src/commands/setup.rs`
- Modify: `src/commands/doctor.rs`
- Modify: `src/commands/config_show.rs`
- Modify: `src/commands/update.rs`
- Modify: `src/commands/uninstall.rs`
- Modify: `src/services/types.rs`
- Test: `tests/services_system.rs`

- [ ] **Step 1: Write the failing service test for setup and uninstall**

```rust
use ctx::services::{runtime::ServiceRuntime, system, types::UninstallRequest};
use tempfile::TempDir;

#[tokio::test]
async fn uninstall_service_preserves_data_without_purge() {
    let data = TempDir::new().unwrap();
    let cache = TempDir::new().unwrap();
    let runtime = ServiceRuntime::bootstrap(
        Some(data.path().to_path_buf()),
        Some(cache.path().to_path_buf()),
        false,
    )
    .await
    .unwrap();

    let result = system::uninstall(
        &runtime,
        UninstallRequest {
            purge_data: false,
        },
    )
    .await
    .unwrap();

    assert!(result.cache_removed);
    assert!(result.data_preserved);
}
```

- [ ] **Step 2: Run the system service test to verify it fails**

Run: `cargo test --test services_system`
Expected: FAIL because `src/services/system.rs` does not exist yet.

- [ ] **Step 3: Define the remaining request and response types**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateRequest {
    pub version: Option<String>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UninstallRequest {
    pub purge_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UninstallResponse {
    pub ok: bool,
    pub cache_removed: bool,
    pub data_removed: bool,
    pub data_preserved: bool,
}
```

- [ ] **Step 4: Move the system behaviors into services and keep the CLI thin**

```rust
pub async fn setup(runtime: &ServiceRuntime, force: bool) -> Result<SetupResponse> {
    runtime.paths.ensure().await?;
    if force || !runtime.paths.config_path.exists() {
        crate::config::save(&runtime.paths, &runtime.config).await?;
    }

    Ok(SetupResponse {
        ok: true,
        data_root: runtime.paths.data_root.display().to_string(),
        cache_root: runtime.paths.cache_root.display().to_string(),
    })
}

pub async fn uninstall(
    runtime: &ServiceRuntime,
    request: UninstallRequest,
) -> Result<UninstallResponse> {
    let cache_removed = if runtime.paths.cache_root.exists() {
        tokio::fs::remove_dir_all(&runtime.paths.cache_root).await.ok();
        true
    } else {
        false
    };

    let data_removed = if request.purge_data && runtime.paths.data_root.exists() {
        tokio::fs::remove_dir_all(&runtime.paths.data_root).await.ok();
        true
    } else {
        false
    };

    Ok(UninstallResponse {
        ok: true,
        cache_removed,
        data_removed,
        data_preserved: !request.purge_data,
    })
}
```

- [ ] **Step 5: Run the system service tests and targeted CLI smoke tests**

Run: `cargo test --test services_system --test cli_smoke`
Expected: PASS with the old command outputs preserved through service-backed adapters.

- [ ] **Step 6: Commit**

```bash
git add src/services/system.rs src/services/types.rs src/commands/setup.rs src/commands/doctor.rs src/commands/config_show.rs src/commands/update.rs src/commands/uninstall.rs tests/services_system.rs
git commit -m "refactor: extract system services"
```

### Task 5: Implement the MCP server core with tools, resources, and prompts

**Files:**
- Create: `src/mcp/mod.rs`
- Create: `src/mcp/server.rs`
- Create: `src/mcp/resources.rs`
- Create: `src/mcp/prompts.rs`
- Modify: `src/lib.rs`
- Test: `tests/mcp_server.rs`

- [ ] **Step 1: Write the failing server contract test**

```rust
use ctx::mcp::server::CtxMcpServer;

#[tokio::test]
async fn mcp_server_advertises_tools_resources_and_prompts() {
    let server = CtxMcpServer::new(None, None).await.unwrap();
    let info = rmcp::ServerHandler::get_info(&server);

    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.resources.is_some());
    assert!(info.capabilities.prompts.is_some());
}
```

- [ ] **Step 2: Run the server contract test to verify it fails**

Run: `cargo test --test mcp_server`
Expected: FAIL because `src/mcp/server.rs` does not exist yet.

- [ ] **Step 3: Add the MCP server struct and tool router**

```rust
use rmcp::{
    ErrorData as McpError, ServerHandler, ServiceExt,
    model::*,
    prompt, prompt_handler, prompt_router,
    tool, tool_handler, tool_router,
};

use crate::services::{
    memory::{self, MemoryAddRequest, MemorySearchRequest},
    runtime::ServiceRuntime,
    system,
};

#[derive(Clone)]
pub struct CtxMcpServer {
    runtime: std::sync::Arc<ServiceRuntime>,
    tool_router: rmcp::handler::server::tool::ToolRouter<Self>,
    prompt_router: rmcp::handler::server::prompt::PromptRouter<Self>,
}

impl CtxMcpServer {
    pub async fn new(
        data_root: Option<std::path::PathBuf>,
        cache_root: Option<std::path::PathBuf>,
    ) -> anyhow::Result<Self> {
        let runtime = ServiceRuntime::bootstrap(data_root, cache_root, false).await?;
        Ok(Self {
            runtime: std::sync::Arc::new(runtime),
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        })
    }
}
```

- [ ] **Step 4: Register typed tools and prompt stubs**

```rust
#[tool_router]
impl CtxMcpServer {
    #[tool(description = "Add text or file content to ctx memory")]
    async fn memory_add(
        &self,
        params: rmcp::handler::server::wrapper::Json<MemoryAddRequest>,
    ) -> Result<rmcp::handler::server::wrapper::Json<serde_json::Value>, McpError> {
        let result = memory::add(&self.runtime, params.0)
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        Ok(rmcp::handler::server::wrapper::Json(result))
    }

    #[tool(description = "Search ctx memory")]
    async fn memory_search(
        &self,
        params: rmcp::handler::server::wrapper::Json<MemorySearchRequest>,
    ) -> Result<rmcp::handler::server::wrapper::Json<crate::search::SearchResult>, McpError> {
        let result = memory::search(&self.runtime, params.0)
            .await
            .map_err(|error| McpError::internal_error(error.to_string(), None))?;
        Ok(rmcp::handler::server::wrapper::Json(result))
    }
}

#[prompt_router]
impl CtxMcpServer {
    #[prompt(description = "Guide a client through first-time ctx setup")]
    async fn first_time_setup(&self) -> Result<GetPromptResult, McpError> {
        Ok(GetPromptResult {
            description: Some("Run setup before ingest or search".into()),
            messages: vec![],
        })
    }
}
```

- [ ] **Step 5: Implement `ServerHandler` for capabilities, resources, and reads**

```rust
#[tool_handler]
#[prompt_handler]
impl ServerHandler for CtxMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Local-first ctx MCP server".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        crate::mcp::resources::list_resources(&self.runtime)
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        crate::mcp::resources::read_resource(&self.runtime, &request.uri)
    }
}
```

- [ ] **Step 6: Run the MCP server test**

Run: `cargo test --test mcp_server`
Expected: PASS with capabilities visible and the server constructible.

- [ ] **Step 7: Commit**

```bash
git add src/lib.rs src/mcp/mod.rs src/mcp/server.rs src/mcp/resources.rs src/mcp/prompts.rs tests/mcp_server.rs
git commit -m "feat: add mcp server core"
```

### Task 6: Wire stdio and HTTP/SSE transports into `ctx mcp serve`

**Files:**
- Modify: `src/commands/mcp.rs`
- Create: `src/mcp/http.rs`
- Create: `src/mcp/stdio.rs`
- Test: `tests/mcp_transports.rs`

- [ ] **Step 1: Write the failing transport tests**

```rust
use std::process::{Command, Stdio};

#[test]
fn mcp_stdio_process_starts() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn stdio server");

    child.kill().ok();
}
```

```rust
#[tokio::test]
async fn mcp_http_starts_on_localhost() {
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_ctx"))
        .args(["mcp", "serve", "--transport", "http", "--host", "127.0.0.1", "--port", "8765"])
        .spawn()
        .expect("spawn http server");

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let _ = child.kill().await;
}
```

- [ ] **Step 2: Run the transport tests to verify they fail**

Run: `cargo test --test mcp_transports`
Expected: FAIL because the command still returns the placeholder error.

- [ ] **Step 3: Implement the `stdio` transport**

```rust
pub async fn serve_stdio(server: crate::mcp::server::CtxMcpServer) -> anyhow::Result<()> {
    let service = server.serve(rmcp::transport::stdio()).await?;
    service.waiting().await?;
    Ok(())
}
```

- [ ] **Step 4: Implement the HTTP/SSE transport wrapper**

```rust
pub async fn serve_http(
    server: crate::mcp::server::CtxMcpServer,
    host: String,
    port: u16,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    let transport = rmcp::transport::streamable_http_server::StreamableHttpService::new(server);
    transport.serve(listener).await?;
    Ok(())
}
```

- [ ] **Step 5: Dispatch the new transport handlers from the CLI command**

```rust
pub async fn run(args: McpServeArgs) -> Result<()> {
    let server = crate::mcp::server::CtxMcpServer::new(None, None).await?;
    match args.transport {
        crate::cli::McpTransport::Stdio => crate::mcp::stdio::serve_stdio(server).await,
        crate::cli::McpTransport::Http => crate::mcp::http::serve_http(server, args.host, args.port).await,
    }
}
```

- [ ] **Step 6: Run the transport tests**

Run: `cargo test --test mcp_transports`
Expected: PASS with both transports startable from the CLI.

- [ ] **Step 7: Commit**

```bash
git add src/commands/mcp.rs src/mcp/http.rs src/mcp/stdio.rs tests/mcp_transports.rs
git commit -m "feat: add mcp transports"
```

### Task 7: Add contract, resource, and prompt regression coverage

**Files:**
- Create: `tests/mcp_contract.rs`
- Modify: `tests/cli_smoke.rs`
- Modify: `README.md`

- [ ] **Step 1: Write the failing contract tests**

```rust
use ctx::mcp::server::CtxMcpServer;
use rmcp::ServerHandler;

#[tokio::test]
async fn mcp_lists_stable_core_tools() {
    let server = CtxMcpServer::new(None, None).await.unwrap();
    let tools = server
        .list_tools(None, rmcp::service::RequestContext::test())
        .await
        .unwrap();

    let names = tools.tools.into_iter().map(|tool| tool.name).collect::<Vec<_>>();
    assert!(names.contains(&"memory_add".to_string()));
    assert!(names.contains(&"memory_search".to_string()));
    assert!(names.contains(&"setup_run".to_string()));
}
```

- [ ] **Step 2: Run the contract test to verify it fails**

Run: `cargo test --test mcp_contract`
Expected: FAIL until the full tool/resource/prompt set is registered.

- [ ] **Step 3: Finish the remaining MCP registrations and document the contract**

```md
## MCP

Start a stdio MCP server:

```bash
ctx mcp serve --transport stdio
```

Start an HTTP/SSE MCP server on localhost:

```bash
ctx mcp serve --transport http --host 127.0.0.1 --port 8765
```

This HTTP transport is local-only in V1. Do not expose it directly to untrusted networks.
```

- [ ] **Step 4: Run the full targeted suite**

Run: `cargo test --test cli_smoke --test services_runtime --test services_memory --test services_system --test mcp_server --test mcp_transports --test mcp_contract`
Expected: PASS with stable MCP names and the CLI still intact.

- [ ] **Step 5: Commit**

```bash
git add README.md tests/mcp_contract.rs tests/cli_smoke.rs
git commit -m "test: lock mcp contract and docs"
```
