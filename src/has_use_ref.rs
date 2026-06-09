use syn::UseTree;
pub fn has_use_ref(entity_name: &str, tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(p) => p.ident == entity_name || has_use_ref(entity_name, &p.tree),
        UseTree::Name(n) => n.ident == entity_name,
        UseTree::Rename(r) => r.ident == entity_name,
        UseTree::Group(g) => g.items.iter().any(|t| has_use_ref(entity_name, t)),
        UseTree::Glob(_) => false,
    }
}
