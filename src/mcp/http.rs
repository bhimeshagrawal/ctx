use std::sync::Arc;

use anyhow::Result;
use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::{SessionManager, local::LocalSessionManager},
};
use serde_json::Value;

use crate::mcp::server::{CtxMcpServer, service_factory};

pub async fn serve(server: CtxMcpServer, host: String, port: u16) -> Result<()> {
    let service = StreamableHttpService::new(
        service_factory(server),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );
    let router = axum::Router::new().route(
        "/mcp",
        any({
            let service = service.clone();
            move |request| {
                let service = service.clone();
                async move { handle_request(service, request).await }
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind((host.as_str(), port)).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

async fn handle_request<S, M>(
    service: StreamableHttpService<S, M>,
    request: Request<Body>,
) -> Response
where
    S: rmcp::Service<rmcp::RoleServer> + Send + 'static,
    M: SessionManager,
{
    let (parts, body) = request.into_parts();

    if parts.method == Method::POST && !parts.headers.contains_key("mcp-session-id") {
        match to_bytes(body, usize::MAX).await {
            Ok(bytes) => {
                if !is_initialize_request(&bytes) {
                    return (StatusCode::BAD_REQUEST, "Bad Request: Session ID is required")
                        .into_response();
                }

                return service
                    .handle(Request::from_parts(parts, Body::from(bytes)))
                    .await
                    .map(Body::new);
            }
            Err(_) => {
                return (StatusCode::BAD_REQUEST, "Bad Request: Failed to read request body")
                    .into_response();
            }
        }
    }

    service.handle(Request::from_parts(parts, body)).await.map(Body::new)
}

fn is_initialize_request(bytes: &[u8]) -> bool {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|payload| {
            payload
                .get("method")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .as_deref()
        == Some("initialize")
}
