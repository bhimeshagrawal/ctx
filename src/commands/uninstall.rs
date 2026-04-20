use anyhow::Result;

use crate::{
    cli::UninstallArgs,
    output,
    paths::CtxPaths,
    services::{system, types::UninstallRequest},
};

pub async fn run(args: UninstallArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let result = system::uninstall(
        &paths,
        UninstallRequest {
            purge_data: args.purge_data,
        },
    )
    .await?;
    output::render(
        &serde_json::json!({
            "ok": result.ok,
            "cacheRemoved": result.cache_removed,
            "dataRemoved": result.data_removed,
            "dataPreserved": result.data_preserved,
        }),
        args.json,
    )
}
