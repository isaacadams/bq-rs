# `bq-rs`

a command line utility for interacting with the bigquery api

## Release

- change version in `Cargo.toml`
- run `bash scripts/release.sh`
  - this will create a git tag based on the version in Cargo.toml and push the tag
- release workflow will kick off, generate cross platform binaries, and put everything into a github release
