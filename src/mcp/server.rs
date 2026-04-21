use std::{borrow::Cow, io, path::PathBuf, sync::Arc};

use anyhow::Result;
use rmcp::{
    handler::server::{
        router::{prompt::PromptRouter, tool::ToolRouter},
        wrapper::Parameters,
    },
    model::{
        GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
        ListResourcesResult, PaginatedRequestParams, PromptMessage, PromptMessageRole,
        ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
    },
    prompt, prompt_handler, prompt_router,
    service::{RequestContext, RoleServer},
    tool, tool_handler, tool_router, Json, ServerHandler,
};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    mcp::resources,
    services::{
        memory::{self, MemoryAddRequest, MemorySearchRequest, MemorySource},
        runtime::ServiceRuntime,
        system,
        types::{UninstallRequest, UpdateRequest},
    },
};

#[derive(Clone)]
pub struct CtxMcpServer {
    runtime: Arc<ServiceRuntime>,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl CtxMcpServer {
    pub async fn new(data_root: Option<PathBuf>, cache_root: Option<PathBuf>) -> Result<Self> {
        let runtime = Arc::new(ServiceRuntime::bootstrap(data_root, cache_root, false).await?);
        Ok(Self {
            runtime,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        })
    }

    pub fn paths(&self) -> &crate::paths::CtxPaths {
        resources::resolve_paths(&self.runtime)
    }

    pub fn runtime(&self) -> &ServiceRuntime {
        self.runtime.as_ref()
    }

    pub fn tool_names(&self) -> Vec<String> {
        self.tool_router
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect()
    }

    pub fn prompt_names(&self) -> Vec<String> {
        self.prompt_router
            .list_all()
            .into_iter()
            .map(|prompt| prompt.name.to_string())
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetupToolRequest {
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAddToolRequest {
    #[serde(default)]
    pub source: Option<MemorySource>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub chunk_size: Option<usize>,
    #[serde(default)]
    pub chunk_overlap: Option<usize>,
}

impl TryFrom<MemoryAddToolRequest> for MemoryAddRequest {
    type Error = String;

    fn try_from(request: MemoryAddToolRequest) -> Result<Self, Self::Error> {
        let source = match (request.source, request.path, request.text) {
            (Some(source), None, None) => source,
            (Some(_), Some(_), _) | (Some(_), _, Some(_)) => {
                return Err(
                    "Provide either `source` or one of `path`/`text`, not both.".to_string()
                );
            }
            (None, Some(path), None) => MemorySource::File { path },
            (None, None, Some(text)) => MemorySource::Text { text },
            (None, Some(_), Some(_)) => {
                return Err("Provide exactly one of `path` or `text`.".to_string());
            }
            (None, None, None) => {
                return Err("Provide one of `path`, `text`, or legacy `source`.".to_string());
            }
        };

        Ok(MemoryAddRequest {
            source,
            title: request.title,
            tags: request.tags,
            chunk_size: request.chunk_size,
            chunk_overlap: request.chunk_overlap,
        })
    }
}

impl JsonSchema for MemoryAddToolRequest {
    fn schema_name() -> Cow<'static, str> {
        "MemoryAddToolRequest".into()
    }

    fn schema_id() -> Cow<'static, str> {
        concat!(module_path!(), "::MemoryAddToolRequest").into()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "object",
            "additionalProperties": false,
            "description": "Add memory from either inline text or a local file path. Provide exactly one of `text` or `path`.",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "Inline text content to ingest."
                },
                "path": {
                    "type": "string",
                    "description": "Absolute or repo-relative path to a local file to ingest."
                },
                "title": {
                    "type": "string",
                    "description": "Optional display title for the document."
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional tag filters stored with the document."
                },
                "chunk_size": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Optional chunk size override."
                },
                "chunk_overlap": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Optional chunk overlap override."
                }
            }
        })
    }
}

#[tool_router]
impl CtxMcpServer {
    #[tool(
        name = "memory_add",
        description = "Add text or file content to ctx memory"
    )]
    async fn memory_add(
        &self,
        Parameters(request): Parameters<MemoryAddToolRequest>,
    ) -> Result<Json<memory::MemoryAddResponse>, String> {
        let request = MemoryAddRequest::try_from(request)?;

        memory::add(&self.runtime, request)
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[tool(name = "memory_search", description = "Search ctx memory")]
    async fn memory_search(
        &self,
        Parameters(request): Parameters<MemorySearchRequest>,
    ) -> Result<Json<crate::search::SearchResult>, String> {
        memory::search(&self.runtime, request)
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[tool(
        name = "setup_run",
        description = "Initialize local ctx paths, config, and storage"
    )]
    async fn setup_run(
        &self,
        Parameters(request): Parameters<SetupToolRequest>,
    ) -> Result<Json<crate::services::types::SetupResponse>, String> {
        system::setup(self.paths(), request.force, false)
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[tool(name = "doctor_run", description = "Run ctx local health checks")]
    async fn doctor_run(&self) -> Result<Json<crate::services::types::DoctorResponse>, String> {
        system::doctor(&self.runtime)
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[tool(
        name = "config_show",
        description = "Show the effective ctx configuration"
    )]
    async fn config_show(&self) -> Result<Json<crate::config::CtxConfig>, String> {
        system::config_show(self.paths())
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[tool(
        name = "update_run",
        description = "Describe the current ctx self-update target"
    )]
    async fn update_run(
        &self,
        Parameters(request): Parameters<UpdateRequest>,
    ) -> Result<Json<crate::update::UpdateDescription>, String> {
        Ok(Json(system::update(
            request,
            std::env::var("CTX_REPO").unwrap_or_else(|_| "bhimeshagrawal/ctx".to_string()),
        )))
    }

    #[tool(
        name = "uninstall_run",
        description = "Remove ctx-managed cache and optionally data"
    )]
    async fn uninstall_run(
        &self,
        Parameters(request): Parameters<UninstallRequest>,
    ) -> Result<Json<crate::services::types::UninstallResponse>, String> {
        system::uninstall(self.paths(), request)
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }
}

#[prompt_router]
impl CtxMcpServer {
    #[prompt(
        name = "memory-add-workflow",
        description = "Guide the client through adding content to ctx memory"
    )]
    async fn memory_add_workflow(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Collect the content source, title, and tags before calling memory_add.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "Use the memory_add tool with either `text` or `path`, plus optional `title`, `tags`, and chunk settings.",
            ),
        ])
        .with_description("Guidance for structured ctx memory ingestion")
    }

    #[prompt(
        name = "memory-search-workflow",
        description = "Guide the client through querying ctx memory"
    )]
    async fn memory_search_workflow(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Choose a concise semantic query and any tag filters before calling memory_search.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "Use memory_search for retrieval, then inspect ctx://config or ctx://status if you need local runtime context.",
            ),
        ])
        .with_description("Guidance for ctx search workflows")
    }

    #[prompt(
        name = "setup-workflow",
        description = "Guide the client through first-time ctx setup"
    )]
    async fn setup_workflow(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Run setup_run before the first ingest or search on a clean machine.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "After setup, read ctx://paths and ctx://config to confirm the managed directories and effective defaults.",
            ),
        ])
        .with_description("Guidance for initial ctx setup")
    }
}

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for CtxMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("ctx", env!("CARGO_PKG_VERSION")))
        .with_instructions(
            "Local-first ctx MCP server with memory, config, diagnostics, and lifecycle tools.",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::ErrorData> {
        Ok(resources::list_resources(&self.runtime))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, rmcp::ErrorData> {
        resources::read_resource(&self.runtime, &request.uri)
            .ok_or_else(|| rmcp::ErrorData::resource_not_found("resource not found", None))
    }
}

pub fn service_factory(
    server: CtxMcpServer,
) -> impl Fn() -> io::Result<CtxMcpServer> + Send + Sync + 'static {
    move || Ok(server.clone())
}
