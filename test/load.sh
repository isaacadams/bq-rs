
# Create a dataset
bq --api http://0.0.0.0:9050 --project_id=test mk test_dataset

# Create a table with schema
bq --api http://0.0.0.0:9050 --project_id=test mk --table test_dataset.test_table name:STRING,age:INTEGER

# load data
bq --api http://0.0.0.0:9050 --project_id=test query < insert_statements.sql
