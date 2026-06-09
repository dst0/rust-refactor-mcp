use syn::UseTree;
pub fn collect_use_names(tree: &UseTree) -> Vec<String> {
    let mut names = Vec::new();
    match tree {
        UseTree::Path(p) => {
            names.extend(collect_use_names(&p.tree));
        }
        UseTree::Name(n) => {
            names.push(n.ident.to_string());
        }
        UseTree::Rename(r) => {
            names.push(r.rename.to_string());
        }
        UseTree::Group(g) => {
            for item in &g.items {
                names.extend(collect_use_names(item));
            }
        }
        UseTree::Glob(_) => {}
    }
    names
}
