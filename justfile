query query service_account: 
  RUST_LOG=info cargo run -- -k {{service_account}} query "{{query}}"

publish: 
  cargo publish -p gauthenticator
  cargo publish

# release workflow will kick off, generate cross platform binaries, and put everything into a github release
release: 
  bash scripts/release.sh

# just delete_version 0.1.6
delete_version tag: 
  git tag --delete {{tag}}

tidy: 
  cargo clippy --verbose --all-features --workspace

clippy-fix: 
  cargo clippy --verbose --all-features --workspace --fix --allow-dirty

# gcloud_directory
# mac: ~/.config/gcloud

list-datasets:
  cargo run -- query "select * from INFORMATION_SCHEMA.SCHEMATA"

list-tables dataset:
  cargo run -- query "select * from `{{dataset}}.INFORMATION_SCHEMA.TABLES`"
