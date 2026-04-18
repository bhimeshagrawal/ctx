use anyhow::Result;

use crate::cli::{MemoryArgs, MemoryCommand};

pub mod add;
pub mod search;

pub async fn run(args: MemoryArgs) -> Result<()> {
    match args.command {
        MemoryCommand::Add(args) => add::run(args).await,
        MemoryCommand::Search(args) => search::run(args).await,
    }
}
