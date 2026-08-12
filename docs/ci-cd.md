# CI/CD

This repository uses GitHub Actions for pull request validation and release publishing.

## PR checks

Workflow file: `.github/workflows/pr-checks.yml`

Runs on:

- `pull_request`
- `merge_group`
- `workflow_dispatch`

Checks performed:

1. `cargo fmt --all -- --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test --locked --all-targets --all-features`
4. `cargo build --locked --release`

Purpose:

- Keep dependency update PRs (including Dependabot) easy to review and merge.
- Ensure style, lint, behavior, and release build continue to pass.

## Release builds and publish

Workflow file: `.github/workflows/release.yml`

Trigger:

- Git tags matching `v*`
- Manual dispatch

Targets:

- Linux: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- macOS: `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`

Artifacts are uploaded and then published to the GitHub Release associated with the tag.

## Dependabot

Dependabot config file: `.github/dependabot.yml`

Update scopes:

- Cargo dependencies
- GitHub Actions versions

Cadence:

- Weekly
