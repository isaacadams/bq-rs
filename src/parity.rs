/// testing this this cli tool achieves parity with googles
/// requires that gcloud be installed
use std::io;
use std::process::Command;

#[cfg(windows)]
const BQ: &str = "bq.cmd";

#[cfg(not(windows))]
const BQ: &str = "bq";

#[test]
fn compare_bq_and_bq_rs_outputs() -> io::Result<()> {
    // Build the Rust project in release mode
    let build_status = Command::new("cargo")
        .args(["build", "--release", "-q"])
        .status()?;
    assert!(build_status.success(), "Cargo build failed!");

    // Execute the `bq` query
    let bq_output = Command::new(BQ)
        .args([
            "--api",
            "http://localhost:9050",
            "query",
            "--project_id=test",
            "--format=csv",
            "SELECT * FROM test_dataset.test_table",
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
            "SELECT * FROM test_dataset.test_table",
        ])
        .output()?;

    assert!(
        bq_rs_output.status.success(),
        "Failed to run `bq-rs` query: {}",
        String::from_utf8_lossy(&bq_rs_output.stderr)
    );

    assert_eq!(
        bq_output.stdout, bq_rs_output.stdout,
        "the outputs of bq bq-rs differ!"
    );

    Ok(())
}
