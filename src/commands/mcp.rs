use anyhow::Result;

use crate::{
    cli::{McpArgs, McpCommand, McpServeArgs, McpTransport},
    mcp::{http, server::CtxMcpServer, stdio},
};

pub async fn run(args: McpArgs) -> Result<()> {
    match args.command {
        McpCommand::Serve(args) => serve(args).await,
    }
}

async fn serve(args: McpServeArgs) -> Result<()> {
    let server = CtxMcpServer::new(None, None).await?;
    match args.transport {
        McpTransport::Stdio => stdio::serve(server).await,
        McpTransport::Http => http::serve(server, args.host, args.port).await,
    }
}
