# use dev.nu
def main [] {
    ^docker compose up -d
}

def "main exec" [cmd: string] {
    ^docker compose exec bigquery-client bash -c ($cmd)
}

def "main bq" [cmd: string] {
    ^docker compose exec bigquery-client bash -c $"bq --api \$HOST --project_id=test ($cmd)"
}

def "main test parity" [] {
    ^cargo b --release -q
    main exec "bq --api \$HOST query --project_id=test --format=csv 'SELECT * FROM test_dataset.test_table' > /test/bq.csv"
    ./target/release/bq-rs --api=http://localhost:9050 --project-id test query "SELECT * FROM test_dataset.test_table" | save -f bq-rs.csv
    ^xxd ./test/bq.csv | save -f bq.hex
    ^xxd bq-rs.csv | save -f bq-rs.hex
    ^code --diff bq-rs.hex bq.hex
    ^md5sum ./test/bq.csv bq-rs.csv
}