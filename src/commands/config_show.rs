use anyhow::Result;

use crate::{cli::ConfigShowArgs, output, paths::CtxPaths, services::system};

pub async fn run(args: ConfigShowArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let config = system::config_show(&paths).await?;
    output::render(&config, args.json)
}
