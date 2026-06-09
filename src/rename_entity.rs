use std::fs;
use std::path::PathBuf;
use syn::{parse_file, visit_mut::VisitMut, Ident};

pub struct Renamer {
    pub old_name: String,
    pub new_name: String,
    pub changed: bool,
}

impl VisitMut for Renamer {
    fn visit_ident_mut(&mut self, i: &mut Ident) {
        if i == &self.old_name {
            *i = Ident::new(&self.new_name, i.span());
            self.changed = true;
        }
    }
}

pub fn rename_entity(file_path: &str, old_name: &str, new_name: &str) -> Result<bool, String> {
    let source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let mut parsed = parse_file(&source).map_err(|e| e.to_string())?;
    let mut renamer = Renamer {
        old_name: old_name.to_string(),
        new_name: new_name.to_string(),
        changed: false,
    };
    renamer.visit_file_mut(&mut parsed);

    if renamer.changed {
        let new_content = prettyplease::unparse(&parsed);
        fs::write(file_path, new_content).map_err(|e| e.to_string())?;

        // Handle file renaming if file matches entity name
        let path = PathBuf::from(file_path);
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            if stem == crate::extract::to_snake_case(old_name) {
                let new_filename = format!("{}.rs", crate::extract::to_snake_case(new_name));
                let new_path = path.with_file_name(new_filename);
                fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
                // Note: Usage updates are complex; for now, we rely on the user or future MCP tooling
                // to handle renaming in usage files.
            }
        }
        let _ = std::process::Command::new("rustfmt")
            .args(["--edition", "2024", file_path])
            .status();
        Ok(true)
    } else {
        Ok(false)
    }
}
