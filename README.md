[![Build Status](https://github.com/primait/bridge.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/primait/bridge.rs)

# bridge.rs

Prima bridge pattern implementation for rust

[Api documentation](https://docs.rs/prima_bridge)

## Examples

You can find all the examples in [examples](./examples) directory.

## Release Process

### TL;DR

**Release a new version:**

1. Merge your PR into `master` and wait for CI to pass.
2. Check the version and notes in the [draft release](https://github.com/primait/bridge.rs/releases).
3. In [Actions > Bump Version and Publish Release](https://github.com/primait/bridge.rs/actions/workflows/bump.yml),
   click **Run workflow**, select `master`, and run it.
4. Wait for [Release on Tags](https://github.com/primait/bridge.rs/actions/workflows/release-on-tags.yml)
   to succeed. The new version is now on crates.io.

**Test a release candidate before merging:**

1. Open a PR for your branch and ensure a draft release exists.
2. In [Actions > Build Release Candidate](https://github.com/primait/bridge.rs/actions/workflows/release-candidate.yml),
   click **Run workflow**, select your PR branch, and run it.
3. Follow the download and Cargo patch instructions posted on your PR, then run
   your consuming project's tests. Nothing is published to crates.io.
4. Remove the temporary Cargo patch and downloaded files when finished.

### How it works

This project uses **[Release Drafter](https://github.com/release-drafter/release-drafter)**
to maintain draft release notes and calculate the next version from PR labels.
The [Release Drafter workflow](./.github/workflows/release-drafter.yml) updates
the draft on pushes to `master` and labels PRs automatically.

### Versioning and labels

The [configuration](./.github/release-drafter.yml) follows the versioning conventions
of `primapy-tracing`, with a separate category for documentation:

- `enhancement` and `bug`: patch bump.
- `breaking change`: minor bump (for the current `0.x` release series).
- `major`: major bump.
- `chore`, `docs`, and `dependencies`: categorized as Maintenance, Documentation,
  and Dependency Updates in the release notes.
- Changes without a version-specific label default to a patch bump.

Dependabot PRs receive `dependencies`; documentation changes receive `docs`,
while workflow changes receive `chore`. A PR body containing `breaking change`
or `BREAKING CHANGE` receives `breaking change`. Check labels before merging;
conventional commit messages alone do not control the release version.

### Publishing a release

1. Merge the changes into `master` and make sure CI passes.
2. Review the single draft in GitHub Releases, including its notes and version.
   New tags use `X.Y.Z`, without a `v` prefix.
3. Run [Bump Version and Publish Release](./.github/workflows/bump.yml) manually
   from `master`.

The workflow updates `Cargo.toml` and `Cargo.lock`, commits and pushes the version
bump, creates the release tag, and publishes the GitHub draft, following the same
flow as `primapy-tracing`.

The tag triggers [Release on Tags](./.github/workflows/release-on-tags.yml), which
runs the existing Rust CI checks and then publishes to crates.io with
`cargo publish --locked`. Check that this workflow succeeds before using the
release: the GitHub release can appear before crates.io publishing finishes.

Publishing uses the existing `CARGO_AUTH_KEY` repository secret. Version commits
and tags use the `DEPLOY_KEY` Actions secret, available to the `release`
environment. The key's identity must be authorized to push directly to protected
`master`, including bypassing the required PR and status checks for the version
commit. SSH alone does not bypass branch protection.
These SSH pushes trigger the CI, release-drafter, and tag-release workflows as
applicable. `GH_TOKEN`, backed by `github.token`, is used for GitHub API calls.

Release notes for new releases live in GitHub Releases; `CHANGELOG.md` retains
the existing history. Do not rerun publishing after a successful upload:
crates.io versions are immutable.

### Testing a release candidate

Run [Build Release Candidate](./.github/workflows/release-candidate.yml) manually
on a branch with exactly one open PR. It uses the draft release version to build
`X.Y.Z-rc.<run number>`, uploads a `.crate` artifact, and creates or updates a PR
comment with download and local Cargo patch instructions.

This workflow does not push a commit or tag and does not publish to crates.io.
For explicitly tagged RC releases, the tag-release workflow also posts the
published dependency version to the associated PR.
