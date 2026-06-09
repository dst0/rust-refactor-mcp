use std::process::Command;
use std::path::Path;

pub fn format_code(path: &str) -> Result<String, String> {
    let status = Command::new("rustfmt")
        .arg(path)
        .status()
        .map_err(|e| format!("Failed to run rustfmt: {}", e))?;

    if status.success() {
        Ok(format!("Successfully formatted {}", path))
    } else {
        Err(format!("rustfmt failed for {}", path))
    }
}
