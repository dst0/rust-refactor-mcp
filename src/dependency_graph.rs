use syn::{visit::Visit, File, Path};
use std::collections::{HashMap, HashSet};

pub struct DependencyGraph {
    pub deps: HashMap<String, HashSet<String>>,
}

impl DependencyGraph {
    pub fn build(parsed: &File) -> Self {
        let mut deps = HashMap::new();
        for item in &parsed.items {
            if let Some(name) = crate::extract::get_item_name(item) {
                let mut visitor = DepVisitor {
                    found: HashSet::new(),
                };
                visitor.visit_item(item);
                deps.insert(name, visitor.found);
            }
        }
        Self { deps }
    }
}

struct DepVisitor {
    found: HashSet<String>,
}

impl<'ast> Visit<'ast> for DepVisitor {
    fn visit_path(&mut self, i: &'ast Path) {
        if let Some(ident) = i.get_ident() {
            self.found.insert(ident.to_string());
        }
        syn::visit::visit_path(self, i);
    }
}
