use std::{io, path::PathBuf, sync::Arc};

use anyhow::Result;
use rmcp::{
    Json, ServerHandler,
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
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    mcp::resources,
    services::{
        memory::{self, MemoryAddRequest, MemorySearchRequest},
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

#[tool_router]
impl CtxMcpServer {
    #[tool(name = "memory_add", description = "Add text or file content to ctx memory")]
    async fn memory_add(
        &self,
        Parameters(request): Parameters<MemoryAddRequest>,
    ) -> Result<Json<memory::MemoryAddResponse>, String> {
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

    #[tool(name = "setup_run", description = "Initialize local ctx paths, config, and storage")]
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

    #[tool(name = "config_show", description = "Show the effective ctx configuration")]
    async fn config_show(&self) -> Result<Json<crate::config::CtxConfig>, String> {
        system::config_show(self.paths())
            .await
            .map(Json)
            .map_err(|error| error.to_string())
    }

    #[tool(name = "update_run", description = "Describe the current ctx self-update target")]
    async fn update_run(
        &self,
        Parameters(request): Parameters<UpdateRequest>,
    ) -> Result<Json<crate::update::UpdateDescription>, String> {
        Ok(Json(system::update(
            request,
            std::env::var("CTX_REPO").unwrap_or_else(|_| "bhimeshagrawal/ctx".to_string()),
        )))
    }

    #[tool(name = "uninstall_run", description = "Remove ctx-managed cache and optionally data")]
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
    #[prompt(name = "memory-add-workflow", description = "Guide the client through adding content to ctx memory")]
    async fn memory_add_workflow(&self) -> GetPromptResult {
        GetPromptResult::new(vec![
            PromptMessage::new_text(
                PromptMessageRole::User,
                "Collect the content source, title, and tags before calling memory_add.",
            ),
            PromptMessage::new_text(
                PromptMessageRole::Assistant,
                "Use the memory_add tool with a structured source object and optional metadata.",
            ),
        ])
        .with_description("Guidance for structured ctx memory ingestion")
    }

    #[prompt(name = "memory-search-workflow", description = "Guide the client through querying ctx memory")]
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

    #[prompt(name = "setup-workflow", description = "Guide the client through first-time ctx setup")]
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
        .with_instructions("Local-first ctx MCP server with memory, config, diagnostics, and lifecycle tools.")
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

pub fn service_factory(server: CtxMcpServer) -> impl Fn() -> io::Result<CtxMcpServer> + Send + Sync + 'static {
    move || Ok(server.clone())
}
