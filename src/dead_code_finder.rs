use std::collections::{HashMap, HashSet};
use std::fs;
use syn::{
    visit::{self, Visit},
    Item,
};

pub struct DeadCodeVisitor {
    pub definitions: HashMap<String, String>, // Name -> Type
    pub references: HashSet<String>,
}

impl<'ast> Visit<'ast> for DeadCodeVisitor {
    fn visit_item(&mut self, i: &'ast Item) {
        let mut is_test = false;
        if let Item::Fn(f) = i {
            if f.attrs.iter().any(|attr| attr.path().is_ident("test")) {
                is_test = true;
            }
        }
        if !is_test {
            if let Some(name) = crate::extract::get_item_name(i) {
                self.definitions
                    .insert(name, crate::extract::item_type(i).to_string());
            }
        }
        visit::visit_item(self, i);
    }

    fn visit_path(&mut self, i: &'ast syn::Path) {
        for segment in &i.segments {
            self.references.insert(segment.ident.to_string());
        }
        visit::visit_path(self, i);
    }
}

pub fn find_dead_code(dir_path: &str) -> Result<String, String> {
    let mut visitor = DeadCodeVisitor {
        definitions: HashMap::new(),
        references: HashSet::new(),
    };

    let mut files = Vec::new();
    crate::split_file::collect_rs_files_internal(std::path::PathBuf::from(dir_path), &mut files);

    for file in files {
        let content = fs::read_to_string(file).map_err(|e| e.to_string())?;
        if let Ok(parsed) = syn::parse_file(&content) {
            visitor.visit_file(&parsed);
        }
    }

    let mut dead_code = Vec::new();
    let excludes = ["main", "cli_main"];
    for (name, itype) in &visitor.definitions {
        if !visitor.references.contains(name) && !excludes.contains(&name.as_str()) {
            dead_code.push(format!("{} ({})", name, itype));
        }
    }

    if dead_code.is_empty() {
        Ok("No dead code found.".to_string())
    } else {
        Ok(format!("Potentially dead code:\n{}", dead_code.join("\n")))
    }
}
