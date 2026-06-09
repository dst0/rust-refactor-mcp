use std::process::Command;

pub fn fix_cargo_errors(path: &str) -> Result<String, String> {
    let status = Command::new("cargo")
        .args(["fix", "--allow-dirty", "--allow-staged", "--manifest-path", path])
        .status()
        .map_err(|e| format!("Failed to run cargo fix: {}", e))?;

    if status.success() {
        Ok(format!("Successfully ran cargo fix on {}", path))
    } else {
        Err(format!("cargo fix failed for {}", path))
    }
}
