use std::fs;
use std::process::Command;
use syn::parse_file;

pub fn optimize_imports(file_path: &str) -> Result<String, String> {
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let parsed = parse_file(&source).map_err(|e| e.to_string())?;

    // Perform AST-based unused import cleanup (reusing logic from extract)
    // Note: We might need a robust way to identify used IDs.
    // For now, let's delegate primary sorting/cleanup to rustfmt,
    // which is powerful, and add a pass to remove clearly unused imports if needed.

    // For now, let's rely on rustfmt for sorting and structure,
    // and implement a simple AST-based cleanup pass.

    let updated_content = prettyplease::unparse(&parsed);
    fs::write(file_path, updated_content).map_err(|e| e.to_string())?;

    let status = Command::new("rustfmt")
        .args(["--edition", "2024", file_path])
        .status()
        .map_err(|e| format!("Failed to run rustfmt: {}", e))?;

    if status.success() {
        Ok(format!("Successfully optimized imports in {}", file_path))
    } else {
        Err(format!("rustfmt failed for {}", file_path))
    }
}
