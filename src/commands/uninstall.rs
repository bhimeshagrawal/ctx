use anyhow::Result;

use crate::{cli::UninstallArgs, output, paths::CtxPaths};

pub async fn run(args: UninstallArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let cache_removed = if paths.cache_root.exists() {
        tokio::fs::remove_dir_all(&paths.cache_root).await.ok();
        true
    } else {
        false
    };

    let data_removed = if args.purge_data && paths.data_root.exists() {
        tokio::fs::remove_dir_all(&paths.data_root).await.ok();
        true
    } else {
        false
    };

    output::render(
        &serde_json::json!({
            "ok": true,
            "cacheRemoved": cache_removed,
            "dataRemoved": data_removed,
            "dataPreserved": !args.purge_data,
        }),
        args.json,
    )
}
