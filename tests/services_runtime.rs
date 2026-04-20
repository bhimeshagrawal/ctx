use ctx::services::runtime::ServiceRuntime;
use tempfile::TempDir;

#[tokio::test]
async fn runtime_bootstrap_uses_explicit_roots() {
    let data = TempDir::new().expect("create data tempdir");
    let cache = TempDir::new().expect("create cache tempdir");

    let runtime = ServiceRuntime::bootstrap(
        Some(data.path().to_path_buf()),
        Some(cache.path().to_path_buf()),
        false,
    )
    .await
    .expect("bootstrap runtime");

    assert_eq!(runtime.paths.data_root, data.path());
    assert_eq!(runtime.paths.cache_root, cache.path());
}
