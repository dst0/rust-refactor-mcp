use std::process::Command;

/// Expand macros for a given item in the crate.
/// Note: Must be run from within a cargo project directory.
pub fn expand_macros(target: &str) -> Result<String, String> {
    let output = Command::new("cargo")
        .args(["expand", target])
        .output()
        .map_err(|e| format!("Failed to run cargo expand: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("cargo expand failed: {}", String::from_utf8_lossy(&output.stderr)))
    }
}
