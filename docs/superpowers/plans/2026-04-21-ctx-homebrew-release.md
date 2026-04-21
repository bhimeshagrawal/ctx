# ctx Homebrew Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move stable `ctx` distribution to Homebrew-only delivery through a separate tap repo, driven by stable git tags and versioned GitHub Releases.

**Architecture:** Keep release metadata and formula rendering in testable Rust helpers, drive stable releases from `vX.Y.Z` tags in GitHub Actions, and update the external tap repo by rewriting `Formula/ctx.rb` from this repo's workflow. Remove the old install script path and rewrite docs around Homebrew as the only supported install method.

**Tech Stack:** Rust, Cargo, GitHub Actions, Homebrew formula, git

---

### Task 1: Add testable release and formula helpers

**Files:**
- Create: `src/release.rs`
- Create: `src/bin/render_homebrew_formula.rs`
- Modify: `src/lib.rs`
- Modify: `tests/release.rs`

- [ ] **Step 1: Write the failing tests for tag parsing and formula rendering**

```rust
use ctx::release::{
    release_archive_name, release_asset_url, render_homebrew_formula, stable_tag_version,
};

#[test]
fn stable_tag_version_accepts_semver_tags() {
    assert_eq!(stable_tag_version("v0.1.0"), Some("0.1.0".to_string()));
}

#[test]
fn stable_tag_version_rejects_non_stable_tags() {
    assert_eq!(stable_tag_version("latest"), None);
    assert_eq!(stable_tag_version("0.1.0"), None);
    assert_eq!(stable_tag_version("v0.1"), None);
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
    assert!(formula.contains("version \"1.2.3\""));
    assert!(formula.contains("sha256 \"abc123\""));
    assert!(formula.contains("releases/download/v1.2.3/ctx-darwin-arm64.tar.gz"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test release`
Expected: FAIL because `ctx::release` and the new helpers do not exist yet.

- [ ] **Step 3: Write the minimal release helper implementation**

```rust
pub fn stable_tag_version(tag: &str) -> Option<String> {
    let version = tag.strip_prefix('v')?;
    let parts = version.split('.').collect::<Vec<_>>();
    if parts.len() != 3 || parts.iter().any(|part| part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit())) {
        return None;
    }
    Some(version.to_string())
}

pub fn release_archive_name(os: &str, arch: &str) -> String {
    format!("ctx-{os}-{arch}.tar.gz")
}

pub fn release_asset_url(repository: &str, tag: &str, os: &str, arch: &str) -> String {
    format!(
        "https://github.com/{repository}/releases/download/{tag}/{}",
        release_archive_name(os, arch)
    )
}

pub fn render_homebrew_formula(repository: &str, version: &str, sha256: &str) -> String {
    format!(
        "class Ctx < Formula\n  desc \"Local-first memory ingest and retrieval CLI\"\n  homepage \"https://github.com/{repository}\"\n  url \"{}\"\n  version \"{version}\"\n  sha256 \"{sha256}\"\n\n  def install\n    bin.install \"ctx\"\n  end\nend\n",
        release_asset_url(repository, &format!(\"v{version}\"), \"darwin\", \"arm64\")
    )
}
```

- [ ] **Step 4: Add the formula renderer binary**

```rust
fn main() {
    let mut repository = None;
    let mut version = None;
    let mut sha256 = None;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repository" => repository = args.next(),
            "--version" => version = args.next(),
            "--sha256" => sha256 = args.next(),
            other => panic!("unexpected argument: {other}"),
        }
    }

    let formula = ctx::release::render_homebrew_formula(
        &repository.expect("missing --repository"),
        &version.expect("missing --version"),
        &sha256.expect("missing --sha256"),
    );
    print!("{formula}");
}
```

- [ ] **Step 5: Wire the module and update the release test file**

```rust
pub mod release;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test --test release`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/release.rs src/bin/render_homebrew_formula.rs src/lib.rs tests/release.rs
git commit -m "feat: add Homebrew release helpers"
```

### Task 2: Switch release automation to stable tags and tap updates

**Files:**
- Modify: `.github/workflows/release.yml`

- [ ] **Step 1: Write the failing workflow assertions into the release test file**

```rust
#[test]
fn stable_tag_version_rejects_latest_tag() {
    assert_eq!(stable_tag_version("latest"), None);
}
```

- [ ] **Step 2: Run tests to verify the release guard behavior fails before the workflow rewrite if helper logic is incomplete**

Run: `cargo test --test release stable_tag_version_rejects_latest_tag`
Expected: PASS only after Task 1 is complete; otherwise FAIL. Use this as the guard before changing the workflow.

- [ ] **Step 3: Replace the release workflow trigger and matrix**

```yaml
on:
  push:
    tags:
      - "v*"
  workflow_dispatch:
```

```yaml
jobs:
  release:
    runs-on: macos-latest
```

- [ ] **Step 4: Add tag-to-Cargo version validation**

```yaml
      - name: Validate release tag
        shell: bash
        run: |
          set -euo pipefail
          TAG="${GITHUB_REF_NAME}"
          VERSION="${TAG#v}"
          CARGO_VERSION="$(grep '^version = ' Cargo.toml | head -n1 | sed -E 's/version = \"([^\"]+)\"/\\1/')"
          test "$TAG" != "$VERSION"
          test "$VERSION" = "$CARGO_VERSION"
          echo "CTX_VERSION=$VERSION" >> "$GITHUB_ENV"
          echo "CTX_TAG=$TAG" >> "$GITHUB_ENV"
```

- [ ] **Step 5: Reduce release packaging to the macOS ARM asset and checksums**

```yaml
      - name: Build release binary
        run: cargo build --release

      - name: Package release artifact
        shell: bash
        run: |
          set -euo pipefail
          mkdir -p release
          cp target/release/ctx .
          tar -czf "release/ctx-darwin-arm64.tar.gz" ctx

      - name: Write checksums
        shell: bash
        run: |
          set -euo pipefail
          cd release
          sha256sum ctx-darwin-arm64.tar.gz > checksums.txt
          SHA="$(awk '/ ctx-darwin-arm64.tar.gz$/ {print $1}' checksums.txt)"
          test -n "$SHA"
          echo "CTX_SHA256=$SHA" >> "$GITHUB_ENV"
```

- [ ] **Step 6: Publish a versioned GitHub Release instead of the rolling latest tag**

```yaml
      - name: Create GitHub release
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ env.CTX_TAG }}
          name: ${{ env.CTX_TAG }}
          target_commitish: ${{ github.sha }}
          files: |
            release/*
```

- [ ] **Step 7: Add the tap repo update step**

```yaml
      - name: Update Homebrew tap
        env:
          HOMEBREW_TAP_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
          HOMEBREW_TAP_REPO: ${{ vars.HOMEBREW_TAP_REPO || format('{0}/homebrew-tap', github.repository_owner) }}
        shell: bash
        run: |
          set -euo pipefail
          test -n "$HOMEBREW_TAP_TOKEN"
          git config user.name "github-actions[bot]"
          git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
          git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/${HOMEBREW_TAP_REPO}.git" tap
          mkdir -p tap/Formula
          cargo run --quiet --bin render_homebrew_formula -- \
            --repository "${GITHUB_REPOSITORY}" \
            --version "${CTX_VERSION}" \
            --sha256 "${CTX_SHA256}" > tap/Formula/ctx.rb
          cd tap
          git add Formula/ctx.rb
          git commit -m "ctx v${CTX_VERSION}"
          git push origin HEAD
```

- [ ] **Step 8: Run focused verification**

Run: `cargo test --test release`
Expected: PASS

- [ ] **Step 9: Commit**

```bash
git add .github/workflows/release.yml tests/release.rs src/release.rs src/bin/render_homebrew_formula.rs src/lib.rs
git commit -m "feat: automate tagged Homebrew releases"
```

### Task 3: Remove the old install path and rewrite docs

**Files:**
- Delete: `install.sh`
- Modify: `README.md`

- [ ] **Step 1: Remove install script references from README**

```md
## Install

```bash
brew tap bhimeshagrawal/homebrew-tap
brew install ctx
```

## Upgrade

```bash
brew upgrade ctx
```
```

- [ ] **Step 2: Rewrite release documentation around stable tags**

```md
## Release

Stable releases are published from version tags such as `v0.1.0`.

The release workflow will:
- verify the tag matches `Cargo.toml`
- build the macOS ARM release artifact
- publish a versioned GitHub Release
- update the Homebrew tap formula automatically
```

- [ ] **Step 3: Delete the install script**

```bash
rm install.sh
```

- [ ] **Step 4: Run focused verification**

Run: `cargo test --test release`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md install.sh
git commit -m "docs: switch installation guidance to Homebrew"
```

### Task 4: Final verification

**Files:**
- Modify: `README.md`
- Modify: `.github/workflows/release.yml`
- Modify: `src/release.rs`
- Modify: `src/mcp/http.rs`
- Modify: `tests/release.rs`
- Modify: `tests/mcp_http_protocol.rs`

- [ ] **Step 1: Run the focused release and MCP verification suite**

Run: `cargo test --test release --test mcp_server --test mcp_contract --test mcp_transports --test mcp_stdio_protocol --test mcp_http_protocol`
Expected: PASS

- [ ] **Step 2: Inspect git status**

Run: `git status --short`
Expected: only intended tracked changes or the unrelated `.claude/` path remain.

- [ ] **Step 3: Commit the final integrated change**

```bash
git add README.md .github/workflows/release.yml src/release.rs src/bin/render_homebrew_formula.rs src/lib.rs tests/release.rs install.sh
git commit -m "feat: publish stable releases through Homebrew tap"
```
