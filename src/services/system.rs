use anyhow::Result;

use crate::{
    config::{self, CtxConfig},
    paths::CtxPaths,
    services::{
        runtime::ServiceRuntime,
        types::{DoctorCheck, DoctorResponse, SetupResponse, UninstallRequest, UninstallResponse, UpdateRequest},
    },
    storage, update,
};

pub async fn setup(paths: &CtxPaths, force: bool, verbose_embeddings: bool) -> Result<SetupResponse> {
    let config = config::load_or_default(paths).await?;

    paths.ensure().await?;
    if force || !paths.config_path.exists() {
        config::save(paths, &config).await?;
    }

    let provider = crate::embeddings::local::LocalEmbeddingProvider::new(
        &config.embeddings.model,
        paths.models_dir.clone(),
        verbose_embeddings,
    );
    storage::init_database(paths, &provider).await?;

    Ok(SetupResponse {
        ok: true,
        data_root: paths.data_root.display().to_string(),
        cache_root: paths.cache_root.display().to_string(),
    })
}

pub async fn doctor(runtime: &ServiceRuntime) -> Result<DoctorResponse> {
    let embedding = runtime.provider.health_check().await;
    let storage_ready = storage::init_database(&runtime.paths, runtime.provider.as_ref()).await.is_ok();

    Ok(DoctorResponse {
        ok: embedding.is_ok() && storage_ready,
        checks: vec![
            DoctorCheck {
                name: "data_root".to_string(),
                ok: runtime.paths.data_root.exists(),
                detail: None,
            },
            DoctorCheck {
                name: "cache_root".to_string(),
                ok: runtime.paths.cache_root.exists(),
                detail: None,
            },
            DoctorCheck {
                name: "embeddings".to_string(),
                ok: embedding.is_ok(),
                detail: Some(embedding.unwrap_or_else(|error| error.to_string())),
            },
            DoctorCheck {
                name: "storage".to_string(),
                ok: storage_ready,
                detail: None,
            },
        ],
    })
}

pub async fn config_show(paths: &CtxPaths) -> Result<CtxConfig> {
    config::load_or_default(paths).await
}

pub fn update(request: UpdateRequest, repository: String) -> update::UpdateDescription {
    let _force = request.force;
    update::describe_update(request.version, repository)
}

pub async fn uninstall(paths: &CtxPaths, request: UninstallRequest) -> Result<UninstallResponse> {
    let cache_removed = if paths.cache_root.exists() {
        tokio::fs::remove_dir_all(&paths.cache_root).await.ok();
        true
    } else {
        false
    };

    let data_removed = if request.purge_data && paths.data_root.exists() {
        tokio::fs::remove_dir_all(&paths.data_root).await.ok();
        true
    } else {
        false
    };

    Ok(UninstallResponse {
        ok: true,
        cache_removed,
        data_removed,
        data_preserved: !request.purge_data,
    })
}
