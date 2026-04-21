use ctx::release::{
    release_archive_name, release_asset_url, release_base_url, render_homebrew_formula,
    stable_tag_version,
};

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
fn release_asset_names_match_expected_archives() {
    assert_eq!(release_archive_name("darwin", "arm64"), "ctx-darwin-arm64.tar.gz");
    assert_eq!(release_archive_name("linux", "x64"), "ctx-linux-x64.tar.gz");
}

#[test]
fn stable_tag_version_accepts_semver_tags() {
    assert_eq!(stable_tag_version("v0.1.0"), Some("0.1.0".to_string()));
    assert_eq!(stable_tag_version("v12.34.56"), Some("12.34.56".to_string()));
}

#[test]
fn stable_tag_version_rejects_non_stable_tags() {
    assert_eq!(stable_tag_version("latest"), None);
    assert_eq!(stable_tag_version("0.1.0"), None);
    assert_eq!(stable_tag_version("v0.1"), None);
    assert_eq!(stable_tag_version("v0.1.0-beta.1"), None);
}

#[test]
fn release_asset_url_uses_versioned_downloads() {
    assert_eq!(
        release_asset_url("owner/repo", "v1.2.3", "darwin", "arm64"),
        "https://github.com/owner/repo/releases/download/v1.2.3/ctx-darwin-arm64.tar.gz"
    );
}

#[test]
fn render_homebrew_formula_uses_versioned_url_and_sha() {
    let formula = render_homebrew_formula("owner/repo", "1.2.3", "abc123");

    assert!(formula.contains("class Ctx < Formula"));
    assert!(formula.contains("homepage \"https://github.com/owner/repo\""));
    assert!(formula.contains("version \"1.2.3\""));
    assert!(formula.contains("sha256 \"abc123\""));
    assert!(formula.contains(
        "https://github.com/owner/repo/releases/download/v1.2.3/ctx-darwin-arm64.tar.gz"
    ));
    assert!(formula.contains("bin.install \"ctx\""));
}
