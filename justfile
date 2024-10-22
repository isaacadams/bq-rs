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

clippy: 
  cargo clippy --verbose --all-features --workspace

clippy-fix:
    cargo clippy --verbose --all-features --workspace --fix --allow-dirty

set-service-account file:
    gcloud auth activate-service-account --key-file="{{file}}"

# adds role to service account
# just service-account-role spot-pet-production data-sftp-share roles/bigquery.transfers.get
service-account-role project name role:
    gcloud projects add-iam-policy-binding {{project}} \
        --member "serviceAccount:{{name}}@{{project}}.iam.gserviceaccount.com" \
        --role {{role}}

test:
  bq --api http://0.0.0.0:9050 query --project_id=test "SELECT * FROM test_dataset.test_table"
  cargo run -- --project-id test query "SELECT * FROM test_dataset.test_table"