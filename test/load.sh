#!/bin/bash
set -e

#export BQ_EMULATOR=1
#export CLOUDSDK_AUTH_ACCESS_TOKEN=dummy

bqlocal() {
    local command=$1
    bq --api $HOST --project_id=test $command
}

# Create a dataset
bqlocal "mk test_dataset"

# Create `test_table`
bqlocal "mk --table test_dataset.test_table name:STRING,age:INTEGER"
bqlocal "query" < /test/insert_statements.sql

# Create `some_empty`
bqlocal "mk --table test_dataset.some_empty name:STRING,age:INTEGER"
bqlocal "query" < /test/some_empty.sql

bqlocal "ls"
bqlocal "show test_dataset.test_table"
bqlocal "show test_dataset.some_empty"
