use syn::{File, Item, ItemFn, ItemUse, Type, UseTree};
pub fn cleanup_unused_imports(source: &str) -> String {
    let Ok(mut parsed) = syn::parse_file(source) else {
        return source.to_string();
    };
    let used = collect_referenced_identifiers(&parsed.items);
    parsed
        .items
        .retain(|item| {
            if let Item::Use(iu) = item {
                let names = collect_use_names(&iu.tree);
                is_import_used(&names, &used)
            } else {
                true
            }
        });
    prettyplease::unparse(&parsed)
}
use crate::extract::{collect_referenced_identifiers, is_import_used, collect_use_names};
