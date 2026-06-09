use crate::collect_use_names::collect_use_names;
use crate::collect_referenced_identifiers::collect_referenced_identifiers;
use syn::{File, Item, ItemUse};
pub fn detect_needed_imports_for_extracted(
    parsed: &File,
    extracted: &[Item],
    _entity_name: &str,
) -> Vec<ItemUse> {
    let used = collect_referenced_identifiers(extracted);
    let mut needed = Vec::new();
    for item in &parsed.items {
        if let Item::Use(iu) = item {
            let names = collect_use_names(&iu.tree);
            let mut include = false;
            if names.iter().any(|n| used.contains(n.as_str()) && n != _entity_name) {
                include = true;
            }
            if !include {
                for name in &names {
                    if name == "Visit" && used.iter().any(|id| id.starts_with("visit_"))
                    {
                        include = true;
                        break;
                    }
                    if name == "VisitMut"
                        && used.iter().any(|id| id.starts_with("visit_"))
                    {
                        include = true;
                        break;
                    }
                    if name == "Spanned" && used.contains("span") {
                        include = true;
                        break;
                    }
                }
            }
            if include {
                needed.push(iu.clone());
            }
        }
    }
    needed
}
