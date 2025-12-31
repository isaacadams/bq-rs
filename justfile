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
  git push origin --delete {{tag}}

set-service-account file:
    gcloud auth activate-service-account --key-file="{{file}}"

# adds role to service account
# just service-account-role spot-pet-production data-sftp-share roles/bigquery.transfers.get
service-account-role project name role:
    gcloud projects add-iam-policy-binding {{project}} \
        --member "serviceAccount:{{name}}@{{project}}.iam.gserviceaccount.com" \
        --role {{role}}

list-datasets:
  cargo run -- query "select * from INFORMATION_SCHEMA.SCHEMATA"

list-tables dataset:
  cargo run -- query "select * from `{{dataset}}.INFORMATION_SCHEMA.TABLES`"
