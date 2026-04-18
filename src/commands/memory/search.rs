use anyhow::{anyhow, Result};

use crate::{
    cli::MemorySearchArgs,
    config,
    embeddings::local::LocalEmbeddingProvider,
    output,
    paths::CtxPaths,
    search,
    storage,
};

pub async fn run(args: MemorySearchArgs) -> Result<()> {
    let query = args.query.join(" ").trim().to_string();
    if query.is_empty() {
        return Err(anyhow!("search query is required"));
    }

    let paths = CtxPaths::resolve(None, None)?;
    let config = config::load_or_default(&paths).await?;
    let provider = LocalEmbeddingProvider::new(&config.embeddings.model, paths.models_dir.clone(), false);
    let db = storage::init_database(&paths, &provider).await?;
    let result = search::run_search(
        &db,
        &provider,
        &config,
        &query,
        args.top_k.unwrap_or(config.defaults.top_k),
        &args.tags,
    )
    .await?;
    output::render(&result, args.json)
}
