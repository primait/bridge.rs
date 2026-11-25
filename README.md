[![Build Status](https://github.com/primait/bridge.rs/actions/workflows/ci.yml/badge.svg)](https://github.com/primait/bridge.rs)

# bridge.rs

Prima bridge pattern implementation for rust

[Api documentation](https://docs.rs/prima_bridge)

## Examples

You can find all the examples in [examples](./examples) directory.

## Release Process

This project uses **[release-plz](https://release-plz.dev/)** for versioning and
publishing to Artifactory and crates.io.

Upon push to `master`, the [CD Workflow](./.github/workflows/cd.yml) is
triggered, which:

- creates or updates a release Pull Request using `release-plz release-pr`
- publishes releases to the `developer-platform-crates` Artifactory registry,
  then to crates.io, using `release-plz release`

This means that to release a new version, you just need to merge the release PR
created by `release-plz` and the rest of the process will be handled
automatically.

Set the `ARTIFACTORY_VAULT_ROLE` repository variable to the Vault JWT role
authorized to read `artifactory/token/<role>` and publish to
`developer-platform-crates`. Until it is set, the release workflow fails before
publishing to either registry. If Artifactory publishing succeeds but publishing
to crates.io fails before the tag is created, rerun the workflow to complete
the release.
Previously tagged versions are not backfilled to Artifactory.

### Conventional Commits

To ensure the correct version bumping, use
[conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) when
squash-merging to `master`. This is not yet enforced by any tool, so it's up to
you to follow the convention.

For breaking changes, it can be useful to include a `BREAKING CHANGE:` section
in the commit body, which will trigger a major version bump. This is especially
important if the breaking change is not obvious from the commit message itself.
