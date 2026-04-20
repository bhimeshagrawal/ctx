use anyhow::Result;
use rmcp::ServiceExt;

use crate::mcp::server::CtxMcpServer;

pub async fn serve(server: CtxMcpServer) -> Result<()> {
    let running = server.serve(rmcp::transport::stdio()).await?;
    let _ = running.waiting().await?;
    Ok(())
}
