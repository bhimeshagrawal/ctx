use anyhow::Result;

use crate::{
    cli::DoctorArgs,
    config,
    embeddings::{self, provider::EmbeddingProvider},
    output,
    paths::CtxPaths,
    storage,
};

pub async fn run(args: DoctorArgs) -> Result<()> {
    let paths = CtxPaths::resolve(None, None)?;
    let config = config::load_or_default(&paths).await?;
    paths.ensure().await?;

    let provider = embeddings::local::LocalEmbeddingProvider::new(
        &config.embeddings.model,
        paths.models_dir.clone(),
        false,
    );

    let embedding = provider.health_check().await;
    let storage_ready = storage::init_database(&paths, &provider).await.is_ok();

    output::render(
        &serde_json::json!({
            "ok": embedding.is_ok() && storage_ready,
            "checks": [
                { "name": "data_root", "ok": paths.data_root.exists() },
                { "name": "cache_root", "ok": paths.cache_root.exists() },
                { "name": "embeddings", "ok": embedding.is_ok(), "detail": embedding.unwrap_or_else(|error| error.to_string()) },
                { "name": "storage", "ok": storage_ready }
            ]
        }),
        args.json,
    )
}
