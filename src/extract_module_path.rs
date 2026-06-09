use syn::UseTree;
pub fn extract_module_path(tree: &UseTree, entity_name: &str) -> String {
    match tree {
        UseTree::Path(p) => {
            if p.ident == entity_name {
                String::new()
            } else {
                let rest = extract_module_path(&p.tree, entity_name);
                if rest.is_empty() {
                    p.ident.to_string()
                } else {
                    format!("{}::{}", p.ident, rest)
                }
            }
        }
        UseTree::Group(g) => {
            for item in &g.items {
                let path = extract_module_path(item, entity_name);
                if !path.is_empty() {
                    return path;
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}
