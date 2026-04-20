use std::sync::Arc;

use anyhow::Result;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use crate::mcp::server::{CtxMcpServer, service_factory};

pub async fn serve(server: CtxMcpServer, host: String, port: u16) -> Result<()> {
    let service = StreamableHttpService::new(
        service_factory(server),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let router = axum::Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
