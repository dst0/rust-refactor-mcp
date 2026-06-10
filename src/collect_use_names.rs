use syn::UseTree;

pub fn collect_use_names(tree: &UseTree) -> Vec<String> {
    collect_use_names_internal(tree, None)
}

fn collect_use_names_internal(tree: &UseTree, last_path: Option<&str>) -> Vec<String> {
    let mut names = Vec::new();
    match tree {
        UseTree::Path(p) => {
            let ident_str = p.ident.to_string();
            names.extend(collect_use_names_internal(&p.tree, Some(&ident_str)));
        }
        UseTree::Name(n) => {
            if n.ident == "self" {
                if let Some(p) = last_path {
                    names.push(p.to_string());
                }
            } else {
                names.push(n.ident.to_string());
            }
        }
        UseTree::Rename(r) => {
            names.push(r.rename.to_string());
        }
        UseTree::Group(g) => {
            for item in &g.items {
                names.extend(collect_use_names_internal(item, last_path));
            }
        }
        UseTree::Glob(_) => {}
    }
    names
}
