use assert_cmd::Command;
use predicates::prelude::*;

/// Helper: returns a Command pointed at the resume-builder binary.
fn cmd() -> Command {
    Command::cargo_bin("resume-builder").expect("resume-builder binary not found")
}

/// Path helpers relative to the workspace root (CARGO_MANIFEST_DIR).
fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name)
}

// ---------------------------------------------------------------------------
// Scenario 1: validate a clean resume — must exit 0 and report success
// ---------------------------------------------------------------------------

#[test]
fn validate_clean_resume_exits_zero() {
    cmd()
        .arg("validate")
        .arg("--resume")
        .arg(fixture("valid_resume.json"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Resume validation passed"));
}

// ---------------------------------------------------------------------------
// Scenario 2: validate an invalid resume — must exit non-zero
//             and report the error message
// ---------------------------------------------------------------------------

#[test]
fn validate_invalid_resume_exits_nonzero() {
    let output = cmd()
        .arg("validate")
        .arg("--resume")
        .arg(fixture("invalid_resume.json"))
        .output()
        .expect("failed to run resume-builder");

    // Binary calls std::process::exit(1) on validation failure, so the
    // process must not have succeeded.
    assert!(
        !output.status.success(),
        "expected non-zero exit for invalid resume, got {:?}",
        output.status
    );

    // The error message is emitted via eprintln! and therefore lives in stderr.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Name is required"),
        "expected 'Name is required' in stderr, got: {}",
        stderr
    );
}

// ---------------------------------------------------------------------------
// Scenario 3: warnings must not cause a non-zero exit
// ---------------------------------------------------------------------------

#[test]
fn validate_warnings_still_exit_zero() {
    // valid_resume.json has no email/phone, which generates ATS warnings.
    // Those warnings must not make the process fail.
    cmd()
        .arg("validate")
        .arg("--resume")
        .arg(fixture("valid_resume.json"))
        .assert()
        .success();
}

// ---------------------------------------------------------------------------
// Scenario 4: keywords subcommand produces non-empty stdout
// ---------------------------------------------------------------------------

#[test]
fn keywords_subcommand_produces_output() {
    cmd()
        .arg("keywords")
        .arg("--resume")
        .arg(fixture("valid_resume.json"))
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

// ---------------------------------------------------------------------------
// Scenario 5: build --format html produces an output file on disk
// ---------------------------------------------------------------------------

#[test]
fn build_html_format_produces_file() {
    let output_dir = "/tmp/resume-cli-test";

    // Remove any leftover artefact from a previous run so the assertion is
    // genuinely testing *this* invocation.
    let _ = std::fs::remove_file(format!("{}/resume.html", output_dir));

    cmd()
        .arg("build")
        .arg("--resume")
        .arg(fixture("valid_resume.json"))
        .arg("--format")
        .arg("html")
        .arg("--output")
        .arg(output_dir)
        .assert()
        .success();

    assert!(
        std::fs::metadata(format!("{}/resume.html", output_dir)).is_ok(),
        "expected {}/resume.html to exist after html build",
        output_dir
    );
}
