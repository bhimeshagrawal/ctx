use anyhow::Result;

use crate::{cli::SetupArgs, config, embeddings, output, paths::CtxPaths, storage};

pub async fn run(args: SetupArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let config = config::load_or_default(&paths).await?;

    paths.ensure().await?;
    if args.force || !paths.config_path.exists() {
        config::save(&paths, &config).await?;
    }

    let provider = embeddings::local::LocalEmbeddingProvider::new(
        &config.embeddings.model,
        paths.models_dir.clone(),
        !args.json,
    );
    storage::init_database(&paths, &provider).await?;
    output::render(
        &serde_json::json!({
            "ok": true,
            "data_root": paths.data_root,
            "cache_root": paths.cache_root,
        }),
        args.json,
    )
}
