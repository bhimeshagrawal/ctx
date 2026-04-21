pub fn stable_tag_version(tag: &str) -> Option<String> {
    let version = tag.strip_prefix('v')?;
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()))
    {
        return None;
    }
    Some(version.to_string())
}

pub fn release_base_url(repository: &str, version: Option<&str>) -> String {
    match version {
        Some(version) => format!("https://github.com/{repository}/releases/download/{version}"),
        None => format!("https://github.com/{repository}/releases/latest/download"),
    }
}

pub fn release_archive_name(os: &str, arch: &str) -> String {
    format!("ctx-{os}-{arch}.tar.gz")
}

pub fn release_asset_url(repository: &str, tag: &str, os: &str, arch: &str) -> String {
    format!(
        "{}/{}",
        release_base_url(repository, Some(tag)),
        release_archive_name(os, arch)
    )
}

pub fn render_homebrew_formula(repository: &str, version: &str, sha256: &str) -> String {
    format!(
        "class Ctx < Formula\n  desc \"Local-first memory ingest and retrieval CLI\"\n  homepage \"https://github.com/{repository}\"\n  url \"{}\"\n  version \"{version}\"\n  sha256 \"{sha256}\"\n\n  def install\n    bin.install \"ctx\"\n  end\nend\n",
        release_asset_url(repository, &format!("v{version}"), "darwin", "arm64")
    )
}
