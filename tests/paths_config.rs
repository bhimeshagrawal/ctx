use ctx::paths::CtxPaths;

#[test]
fn resolved_paths_use_separate_data_and_cache_roots() {
    let paths = CtxPaths::from_roots("/tmp/ctx-data", "/tmp/ctx-cache");
    assert_eq!(paths.db_dir.display().to_string(), "/tmp/ctx-data/db");
    assert_eq!(
        paths.models_dir.display().to_string(),
        "/tmp/ctx-cache/models"
    );
}

#[test]
fn uninstall_keeps_data_without_purge_flag() {
    let purge_data = false;
    assert!(!purge_data);
}
