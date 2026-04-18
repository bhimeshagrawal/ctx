use ctx::update::{release_archive_name, release_base_url};

#[test]
fn release_base_url_uses_latest_by_default() {
    assert_eq!(
        release_base_url("owner/repo", None),
        "https://github.com/owner/repo/releases/latest/download"
    );
    assert_eq!(
        release_base_url("owner/repo", Some("v1.2.3")),
        "https://github.com/owner/repo/releases/download/v1.2.3"
    );
}

#[test]
fn release_asset_names_match_install_script() {
    assert_eq!(release_archive_name("darwin", "arm64"), "ctx-darwin-arm64.tar.gz");
    assert_eq!(release_archive_name("linux", "x64"), "ctx-linux-x64.tar.gz");
}
