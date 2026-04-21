use rmcp::model::{
    Annotated, ListResourcesResult, RawResource, ReadResourceResult, ResourceContents,
};

use crate::{paths::CtxPaths, services::runtime::ServiceRuntime};

pub const CONFIG_URI: &str = "ctx://config";
pub const PATHS_URI: &str = "ctx://paths";
pub const STATUS_URI: &str = "ctx://status";

pub fn list_resources(_runtime: &ServiceRuntime) -> ListResourcesResult {
    ListResourcesResult::with_all_items(vec![
        Annotated::new(
            RawResource::new(CONFIG_URI, "ctx-config")
                .with_description("Effective ctx configuration")
                .with_mime_type("application/json"),
            None,
        ),
        Annotated::new(
            RawResource::new(PATHS_URI, "ctx-paths")
                .with_description("Resolved managed ctx paths")
                .with_mime_type("application/json"),
            None,
        ),
        Annotated::new(
            RawResource::new(STATUS_URI, "ctx-status")
                .with_description("Current ctx runtime status")
                .with_mime_type("application/json"),
            None,
        ),
    ])
}

pub fn read_resource(runtime: &ServiceRuntime, uri: &str) -> Option<ReadResourceResult> {
    let content = match uri {
        CONFIG_URI => serde_json::to_string_pretty(&runtime.config)
            .ok()
            .map(|json| {
                ResourceContents::text(json, CONFIG_URI).with_mime_type("application/json")
            }),
        PATHS_URI => serde_json::to_string_pretty(&runtime.paths)
            .ok()
            .map(|json| ResourceContents::text(json, PATHS_URI).with_mime_type("application/json")),
        STATUS_URI => {
            let value = serde_json::json!({
                "dataRoot": runtime.paths.data_root,
                "cacheRoot": runtime.paths.cache_root,
                "dbDir": runtime.paths.db_dir,
                "configPath": runtime.paths.config_path,
            });
            Some(
                ResourceContents::text(serde_json::to_string_pretty(&value).ok()?, STATUS_URI)
                    .with_mime_type("application/json"),
            )
        }
        _ => None,
    }?;

    Some(ReadResourceResult::new(vec![content]))
}

pub fn resolve_paths(runtime: &ServiceRuntime) -> &CtxPaths {
    &runtime.paths
}
