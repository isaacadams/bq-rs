HOST=http://localhost:9050

# Create a dataset
bq --api $HOST --project_id=test mk test_dataset

# Create a table with schema
bq --api $HOST --project_id=test mk --table test_dataset.test_table name:STRING,age:INTEGER

# load data
bq --api $HOST --project_id=test query < ./test/insert_statements.sql
