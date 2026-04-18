use anyhow::Result;
use clap::Parser;
use ctx::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
