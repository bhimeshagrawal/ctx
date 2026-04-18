use anyhow::Result;

use crate::cli::{ConfigArgs, ConfigCommand};

pub async fn run(args: ConfigArgs) -> Result<()> {
    match args.command {
        ConfigCommand::Show(args) => crate::commands::config_show::run(args).await,
    }
}
