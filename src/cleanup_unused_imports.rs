use crate::collect_use_names::collect_use_names;
use crate::collect_referenced_identifiers::collect_referenced_identifiers;
use crate::is_import_used::is_import_used;
use syn::Item;
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
