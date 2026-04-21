use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use crate::commands;

#[derive(Debug, Parser)]
#[command(
    name = "ctx",
    version,
    about = "Local-first memory ingest and retrieval CLI"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        match self.command {
            Commands::Setup(args) => commands::setup::run(args).await,
            Commands::Uninstall(args) => commands::uninstall::run(args).await,
            Commands::Doctor(args) => commands::doctor::run(args).await,
            Commands::Update(args) => commands::update::run(args).await,
            Commands::Config(args) => commands::config::run(args).await,
            Commands::Memory(args) => commands::memory::run(args).await,
            Commands::Mcp(args) => commands::mcp::run(args).await,
        }
    }
}

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
pub struct SetupArgs {
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Debug, Args)]
pub struct UninstallArgs {
    #[arg(long, default_value_t = false)]
    pub force: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long = "purge-data", default_value_t = false)]
    pub purge_data: bool,
}

#[derive(Debug, Args)]
pub struct DoctorArgs {
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct UpdateArgs {
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, default_value_t = false)]
    pub force: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Show(ConfigShowArgs),
}

#[derive(Debug, Args)]
pub struct ConfigShowArgs {
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct MemoryArgs {
    #[command(subcommand)]
    pub command: MemoryCommand,
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    Add(MemoryAddArgs),
    Search(MemorySearchArgs),
}

#[derive(Debug, Args)]
pub struct MemoryAddArgs {
    #[arg(long)]
    pub file: Option<String>,
    #[arg(long)]
    pub text: Option<String>,
    #[arg(long, default_value_t = false)]
    pub stdin: bool,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long = "chunk-size")]
    pub chunk_size: Option<usize>,
    #[arg(long = "chunk-overlap")]
    pub chunk_overlap: Option<usize>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct MemorySearchArgs {
    pub query: Vec<String>,
    #[arg(long = "top-k")]
    pub top_k: Option<usize>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long, default_value_t = false)]
    pub raw: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
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

#[derive(Clone, Debug, clap::ValueEnum)]
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
    #[arg(long, default_value_t = 8765)]
    pub port: u16,
}
