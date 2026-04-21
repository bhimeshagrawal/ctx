# ctx Homebrew Release Design

## Goal

Move stable `ctx` distribution to Homebrew only.

The release source remains this repository. Homebrew distribution is published through a separate tap repository that is updated automatically after each stable tagged release.

## Decisions

- Stable publishing is triggered only by git tags matching `vX.Y.Z`.
- `Cargo.toml` is the canonical in-repo version and must exactly match the pushed git tag without the `v` prefix.
- End-user installation is Homebrew only.
- The curl-based install script is removed.
- V1 Homebrew distribution supports macOS ARM only.
- GitHub Releases continue to hold versioned release artifacts because the tap formula needs a stable URL and SHA256 source.
- The tap lives in a separate repository, expected by default to be `bhimeshagrawal/homebrew-tap`.
- This repository's GitHub Actions workflow updates the tap repository directly using a token secret.

## Release Model

The release workflow should no longer run on pushes to `main`. It should run on stable version tags only, plus optional manual dispatch for testing.

For each stable tag:

1. Validate the tag format.
2. Read `Cargo.toml` and fail unless `package.version` matches the tag without the `v`.
3. Build the macOS ARM release artifact.
4. Write `checksums.txt`.
5. Publish a versioned GitHub Release for that tag.
6. Update the Homebrew tap formula with the new versioned asset URL and SHA256.

The rolling `latest` tag is removed from the release process.

## Artifact Shape

V1 publishes a single release asset:

- `ctx-darwin-arm64.tar.gz`

The formula should reference:

- `https://github.com/bhimeshagrawal/ctx/releases/download/vX.Y.Z/ctx-darwin-arm64.tar.gz`

The release workflow should fail if the artifact checksum cannot be resolved from `checksums.txt`.

## Tap Update Model

The tap repository stores `Formula/ctx.rb`.

After the GitHub Release is published, the workflow clones the tap repo, rewrites `Formula/ctx.rb`, commits a message like `ctx vX.Y.Z`, and pushes to the tap default branch.

The formula should contain:

- the exact `version`
- the versioned `url`
- the release `sha256`
- `bin.install "ctx"`

No HEAD formula is needed.

## Repository Changes

This repository should:

- remove `install.sh`
- add a deterministic formula renderer for workflow automation
- keep release helper logic in testable Rust code
- update README installation and release sections to Homebrew-only guidance

The CLI may retain `ctx update` internally for now, but it is no longer the documented upgrade path. Documentation should instruct users to run `brew upgrade ctx`.

## Verification

Required verification:

- unit tests for stable tag parsing and formula rendering
- release helper tests for versioned URLs and artifact naming
- workflow validation by running the Rust test suite locally

Operational failure rules:

- if build or release publishing fails, tap update does not run
- if tap update fails, the workflow fails loudly and can be rerun
- formula generation must be deterministic from repository slug, version, and checksum
