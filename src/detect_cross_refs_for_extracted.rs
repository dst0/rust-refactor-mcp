use crate::collect_referenced_identifiers::collect_referenced_identifiers;
use std::path::PathBuf;
use syn::{File, Item, ItemUse};
pub fn detect_cross_refs_for_extracted(
    parsed: &File,
    extracted: &[Item],
    entity_name: &str,
    source_file_path: Option<&str>,
) -> Vec<ItemUse> {
    let defined: Vec<String> = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Struct(s) if s.ident != entity_name => Some(s.ident.to_string()),
            Item::Enum(e) if e.ident != entity_name => Some(e.ident.to_string()),
            Item::Trait(t) if t.ident != entity_name => Some(t.ident.to_string()),
            Item::Fn(f) if f.sig.ident != entity_name => Some(f.sig.ident.to_string()),
            _ => None,
        })
        .collect();
    let used = collect_referenced_identifiers(extracted);
    let needed: Vec<String> = defined
        .into_iter()
        .filter(|n| used.contains(n.as_str()))
        .collect();
    if needed.is_empty() {
        return Vec::new();
    }
    let module_name = source_file_path
        .and_then(|p| {
            PathBuf::from(p).file_stem().and_then(|s| s.to_str().map(|s| s.to_string()))
        })
        .unwrap_or_else(|| "super".to_string());
    let names = needed.join(", ");
    let use_str = format!("use crate::{}::{{{}}};", module_name, names);
    let Ok(parsed) = syn::parse_file(&use_str) else {
        return Vec::new();
    };
    parsed
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Use(iu) => Some(iu),
            _ => None,
        })
        .collect()
}
