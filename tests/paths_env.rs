use ctx::paths::CtxPaths;

#[test]
fn empty_env_overrides_are_ignored() {
    std::env::set_var("CTX_DATA_DIR", "");
    std::env::set_var("CTX_CACHE_DIR", "");
    let paths = CtxPaths::resolve(None, None).expect("resolve paths");
    assert!(paths.data_root.is_absolute());
    assert!(paths.cache_root.is_absolute());
}
