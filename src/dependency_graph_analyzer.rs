use std::collections::{HashMap, HashSet};
use std::fs;
use syn::visit::{self, Visit};

pub struct GraphAnalyzer<'a> {
    pub current_mod: String,
    pub deps: HashSet<String>,
    pub all_modules: &'a HashSet<String>,
}

impl<'ast, 'a> Visit<'ast> for GraphAnalyzer<'a> {
    fn visit_path(&mut self, i: &'ast syn::Path) {
        for segment in &i.segments {
            let name = segment.ident.to_string();
            if self.all_modules.contains(&name) && name != self.current_mod {
                self.deps.insert(name);
            }
        }
        visit::visit_path(self, i);
    }
}

pub fn analyze_dependencies(dir_path: &str) -> Result<String, String> {
    let mut files = Vec::new();
    crate::split_file::collect_rs_files_internal(std::path::PathBuf::from(dir_path), &mut files);

    let mut all_modules = HashSet::new();
    for file in &files {
        if let Some(stem) = file.file_stem() {
            all_modules.insert(stem.to_string_lossy().to_string());
        }
    }

    let mut module_deps: HashMap<String, HashSet<String>> = HashMap::new();

    for file in files {
        if let Some(stem) = file.file_stem() {
            let mod_name = stem.to_string_lossy().to_string();
            let content = fs::read_to_string(&file).map_err(|e| e.to_string())?;
            if let Ok(parsed) = syn::parse_file(&content) {
                let mut visitor = GraphAnalyzer {
                    current_mod: mod_name.clone(),
                    deps: HashSet::new(),
                    all_modules: &all_modules,
                };
                visitor.visit_file(&parsed);
                if !visitor.deps.is_empty() {
                    module_deps.insert(mod_name, visitor.deps);
                }
            }
        }
    }

    if module_deps.is_empty() {
        return Ok("No internal dependencies found.".to_string());
    }

    let mut result = String::new();
    result.push_str("Internal Dependency Graph:\n");
    let mut sorted_mods: Vec<_> = module_deps.keys().collect();
    sorted_mods.sort();

    for m in sorted_mods {
        let deps = &module_deps[m];
        let mut sorted_deps: Vec<_> = deps.iter().collect();
        sorted_deps.sort();
        result.push_str(&format!("  {} -> {:?}\n", m, sorted_deps));
    }

    Ok(result)
}
