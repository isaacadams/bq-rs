# `bq-rs`

a command line utility for interacting with the bigquery api

## Commands

### Authenticate w/ Service Account

All the commands require authentication via service account. The path to the service account key json file can be passed into `bq-rs` via the `--key` argument.

#### `bq-rs --key <SERVICE-ACCOUNT-KEY-PATH> ...`

e.g. `bq-rs --key ./key.json ...`

### Query

Bigquery tables can be queried and its results returned as CSV by using the `query` subcommand.

#### `bq-rs <...ARGS> query <QUERY>`

e.g. `bq-rs --key ./key.json query "SELECT * FROM <project-id>.<dataset-id>.<table-id>"`
