use std::process::Command;

pub fn validate_project(manifest_path: &str) -> Result<String, String> {
    let mut report = String::new();

    // 1. Run cargo check
    let check_status = Command::new("cargo")
        .args([
            "check",
            "--manifest-path",
            manifest_path,
            "--color",
            "never",
        ])
        .output()
        .map_err(|e| format!("Failed to run cargo check: {}", e))?;

    if !check_status.status.success() {
        return Err(format!(
            "cargo check failed:\n{}",
            String::from_utf8_lossy(&check_status.stderr)
        ));
    }
    report.push_str("cargo check passed.\n");

    // 2. Run cargo test
    let test_status = Command::new("cargo")
        .args(["test", "--manifest-path", manifest_path, "--color", "never"])
        .output()
        .map_err(|e| format!("Failed to run cargo test: {}", e))?;

    if !test_status.status.success() {
        return Err(format!(
            "cargo test failed:\n{}",
            String::from_utf8_lossy(&test_status.stderr)
        ));
    }
    report.push_str("cargo test passed.\n");

    Ok(report)
}
