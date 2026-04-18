use anyhow::Result;

use crate::{cli::ConfigShowArgs, config, output, paths::CtxPaths};

pub async fn run(args: ConfigShowArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let config = config::load_or_default(&paths).await?;
    output::render(&config, args.json)
}
