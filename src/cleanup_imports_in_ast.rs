use crate::collect_use_names::collect_use_names;
use crate::is_import_used::is_import_used;
use std::collections::HashSet;
use syn::{File, Item};
pub fn cleanup_imports_in_ast(parsed: &File, used_ids: &HashSet<String>) -> File {
    let mut cleaned = parsed.clone();
    cleaned
        .items
        .retain(|item| {
            if let Item::Use(iu) = item {
                let names = collect_use_names(&iu.tree);
                is_import_used(&names, used_ids)
            } else {
                true
            }
        });
    cleaned
}
