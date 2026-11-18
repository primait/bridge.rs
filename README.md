[![Build Status](https://github.com/primait/bridge.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/primait/bridge.rs)

# bridge.rs

Prima bridge pattern implementation for rust

[Api documentation](https://docs.rs/prima_bridge)

## Examples

You can find all the examples in [examples](./examples) directory.

## Release Process

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
bump, publishes to crates.io, creates the release tag, and publishes the GitHub
draft. It rejects invalid versions, downgrades, and existing tags (including
legacy `vX.Y.Z` tags). Release drafting and publishing are serialized.

Publishing uses the existing `CARGO_AUTH_KEY` repository secret. Version commits
and tags use the `SSH_KEY` Actions secret, whose identity must be authorized to
push directly to protected `master`, including bypassing the required PR and
status checks for the version commit. SSH alone does not bypass branch protection.
These pushes can trigger other workflows; crates.io publishing still runs in
the same workflow and verifies the package with Cargo. `GH_TOKEN`, backed by
`github.token`, is used only for GitHub API calls to read and publish the draft.

Release notes for new releases live in GitHub Releases; `CHANGELOG.md` retains
the existing history. If a run fails after the crate has been published, finish
the tag and draft publication manually rather than rerunning `cargo publish`:
crates.io versions are immutable.
