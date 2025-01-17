export BQ_EMULATOR=1
#HOST=http://localhost:9050
export CLOUDSDK_AUTH_ACCESS_TOKEN=dummy

# Create a dataset
bq --api $HOST --project_id=test mk test_dataset

# Create a table with schema
bq --api $HOST --project_id=test mk --table test_dataset.test_table name:STRING,age:INTEGER

# load data
bq --api $HOST --project_id=test query < ./test/insert_statements.sql

# Create `some_empty``
bq --api $HOST --project_id=test mk --table test_dataset.some_empty name:STRING,age:INTEGER
bq --api $HOST --project_id=test query < ./test/some_empty.sql
