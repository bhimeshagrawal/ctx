use anyhow::Result;

use crate::{cli::SetupArgs, output, paths::CtxPaths, services::system};

pub async fn run(args: SetupArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let result = system::setup(&paths, args.force, !args.json).await?;
    output::render(&result, args.json)
}
