/// testing this this cli tool achieves parity with googles
/// requires that gcloud be installed
/// it also expects a specific dataset to be loaded
/// ensure to run `just dev` before running these tests
use std::io;
use std::process::Command;

#[cfg(windows)]
const BQ: &str = "bq.cmd";

#[cfg(windows)]
const DIGEST: &str = "cce3a33e8e72c2ce410c3ef9094800e7";

#[cfg(not(windows))]
const BQ: &str = "bq";

#[cfg(not(windows))]
const DIGEST: &str = "00d13773641c34a4f0fd5ed24f2a2986";

/// Build the Rust project in release mode
fn release() -> io::Result<()> {
    let build_status = Command::new("cargo")
        .args(["build", "--release", "-q"])
        .status()?;
    assert!(build_status.success(), "Cargo build failed!");
    Ok(())
}

#[test]
fn should_match_expected_md5() -> io::Result<()> {
    release()?;

    // Execute the `bq-rs` query
    let bq_rs_output = Command::new("./target/release/bq-rs")
        .args([
            "--api=http://localhost:9050",
            "--project-id=test",
            "query",
            "SELECT * FROM test_dataset.test_table",
        ])
        .output()?;

    assert!(
        bq_rs_output.status.success(),
        "Failed to run `bq-rs` query: {}",
        String::from_utf8_lossy(&bq_rs_output.stderr)
    );

    //md5::compute(bq_rs_output.stdout).as_ref();

    assert_eq!(
        format!("{:#?}", md5::compute(&bq_rs_output.stdout)).as_str(),
        DIGEST,
        "md5 digest is not correct"
    );

    Ok(())
}

#[test]
fn compare_csv_outputs() -> io::Result<()> {
    compare_bq_and_bq_rs_csv_outputs("SELECT * FROM test_dataset.test_table")?;
    compare_bq_and_bq_rs_csv_outputs("SELECT * FROM test_dataset.some_empty")?;
    Ok(())
}

fn compare_bq_and_bq_rs_csv_outputs(query: &str) -> io::Result<()> {
    release()?;

    // Execute the `bq` query
    let bq_output = Command::new(BQ)
        .args([
            "--api",
            "http://localhost:9050",
            "query",
            "--project_id=test",
            "--format=csv",
            query,
        ])
        .output()?;

    assert!(
        bq_output.status.success(),
        "Failed to run `bq` query: {}",
        String::from_utf8_lossy(&bq_output.stderr)
    );

    // Execute the `bq-rs` query
    let bq_rs_output = Command::new("./target/release/bq-rs")
        .args([
            "--api=http://localhost:9050",
            "--project-id=test",
            "query",
            query,
        ])
        .output()?;

    assert!(
        bq_rs_output.status.success(),
        "Failed to run `bq-rs` query: {}",
        String::from_utf8_lossy(&bq_rs_output.stderr)
    );

    assert_eq!(
        bq_output.stdout, bq_rs_output.stdout,
        "\n\nthe outputs of bq bq-rs differ for the query:\n\n```\n{}\n```\n\nbq:\n```\n{}\n```\n\nbq-rs:\n```\n{}\n```\n\n",
        query,
        String::from_utf8_lossy(&bq_output.stdout),
        String::from_utf8_lossy(&bq_rs_output.stdout)
    );

    Ok(())
}
