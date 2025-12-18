# use dev.nu
def main [] {
    #gcloud config set component_manager/disable_update_check true
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
    
    #main exec "bq --api \$HOST query --project_id=test --format=csv 'SELECT * FROM test_dataset.test_table' > /test/bq.csv"

    mkdir parity
    let bq_path = "parity" | path join "bq.csv"
    let bq_rs_path = "parity" | path join "bq-rs.csv"
    let bq_hex_path = "parity" | path join "bq.hex"
    let bq_rs_hex_path = "parity" | path join "bq-rs.hex"

    main bqq 'SELECT * FROM test_dataset.test_table' | save -f $bq_path
    ./target/release/bq-rs --api=http://localhost:9050 --project-id test query "SELECT * FROM test_dataset.test_table" | save -f $bq_rs_path
    ^xxd $bq_path | save -f $bq_hex_path
    ^xxd $bq_rs_path | save -f $bq_rs_hex_path
    ^code --diff $bq_rs_hex_path $bq_hex_path
    ^md5sum $bq_path $bq_rs_path
}

def "main tidy" [--d] {
    if $d {
        ^cargo clippy --verbose --all-features --workspace --fix --allow-dirty
    } else {
        ^cargo clippy --verbose --all-features --workspace
    }
}

def "main bqq" [query: string] {
    with-env {
        HOST: "http://localhost:9050"
        BQ_EMULATOR: 1
        CLOUDSDK_AUTH_ACCESS_TOKEN: "dummy"
    } {
        ^bq --api $env.HOST --project_id=test query --format csv $query 
            | lines 
            | skip 1
            | str join "\n"
            | from csv
    }
}

# nu dev.nu query "SELECT * FROM INFORMATION_SCHEMA.SCHEMATA"
# nu dev.nu query "SELECT * FROM `<dataset_id>.INFORMATION_SCHEMA.TABLES`"
# e.g. nu dev.nu query "SELECT * FROM `spot-pet-production.Board_Deck.INFORMATION_SCHEMA.TABLES`"
# nu dev.nu query "SELECT * FROM spot-pet-production.reference_files.all_time_eligible_quoters_query limit 10"
def "main query" [query: string] {
    with-env {
        RUST_LOG: "debug"
    } {
        cargo run -- -k ./credentials/key.json query $query
    }
}