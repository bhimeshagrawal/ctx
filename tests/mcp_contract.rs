use ctx::mcp::{resources, server::CtxMcpServer};
use tempfile::TempDir;

#[tokio::test]
async fn mcp_contract_exposes_stable_core_names() {
    let data = TempDir::new().expect("create data tempdir");
    let cache = TempDir::new().expect("create cache tempdir");
    let server = CtxMcpServer::new(
        Some(data.path().to_path_buf()),
        Some(cache.path().to_path_buf()),
    )
    .await
    .expect("create mcp server");

    let tool_names = server.tool_names();
    let prompt_names = server.prompt_names();
    let resource_names = resources::list_resources(server.runtime())
        .resources
        .into_iter()
        .map(|resource| resource.raw.uri)
        .collect::<Vec<_>>();

    assert!(tool_names.contains(&"memory_add".to_string()));
    assert!(tool_names.contains(&"memory_search".to_string()));
    assert!(tool_names.contains(&"setup_run".to_string()));
    assert!(tool_names.contains(&"doctor_run".to_string()));
    assert!(tool_names.contains(&"config_show".to_string()));
    assert!(tool_names.contains(&"update_run".to_string()));
    assert!(tool_names.contains(&"uninstall_run".to_string()));

    assert!(prompt_names.contains(&"memory-add-workflow".to_string()));
    assert!(prompt_names.contains(&"memory-search-workflow".to_string()));
    assert!(prompt_names.contains(&"setup-workflow".to_string()));

    assert!(resource_names.contains(&resources::CONFIG_URI.to_string()));
    assert!(resource_names.contains(&resources::PATHS_URI.to_string()));
    assert!(resource_names.contains(&resources::STATUS_URI.to_string()));
}
