use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Serialize, JsonSchema)]
pub struct UpdateDescription {
    pub repository: String,
    pub version: Option<String>,
    pub base_url: String,
    pub archive: String,
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

pub fn describe_update(version: Option<String>, repository: String) -> UpdateDescription {
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        other => other,
    };

    UpdateDescription {
        base_url: release_base_url(&repository, version.as_deref()),
        archive: release_archive_name(os, arch),
        repository,
        version,
    }
}
