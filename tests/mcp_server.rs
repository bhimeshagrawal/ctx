use ctx::mcp::server::CtxMcpServer;
use rmcp::ServerHandler;
use tempfile::TempDir;

#[tokio::test]
async fn mcp_server_advertises_tools_prompts_and_resources() {
    let data = TempDir::new().expect("create data tempdir");
    let cache = TempDir::new().expect("create cache tempdir");
    let server = CtxMcpServer::new(
        Some(data.path().to_path_buf()),
        Some(cache.path().to_path_buf()),
    )
    .await
    .expect("create mcp server");

    let info = server.get_info();

    assert!(info.capabilities.tools.is_some());
    assert!(info.capabilities.prompts.is_some());
    assert!(info.capabilities.resources.is_some());
}
