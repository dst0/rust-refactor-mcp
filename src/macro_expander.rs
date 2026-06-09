use std::process::Command;

/// Expand macros for a given target in the crate.
/// 
/// The target can be a specific item (e.g., `my_module::my_function`) 
/// or a file path depending on how `cargo-expand` is configured.
pub fn expand_macros(target: &str) -> Result<String, String> {
    // Attempt to run cargo expand.
    // Ensure we are in a cargo-managed directory.
    let output = Command::new("cargo")
        .args(["expand", target, "--color", "never"])
        .output()
        .map_err(|e| format!("Failed to execute cargo expand: {}. Ensure cargo-expand is installed (`cargo install cargo-expand`).", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            Ok("Macro expansion produced no output.".to_string())
        } else {
            Ok(stdout.to_string())
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("cargo expand failed:\n{}", stderr))
    }
}
