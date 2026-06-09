use crate::format_ty_name::format_ty_name;
use crate::namevisitor::NameVisitor;
use std::collections::HashSet;
use syn::{File, Item};
pub fn find_extracted_indices(parsed: &File, entity_name: &str) -> HashSet<usize> {
    let mut indices = HashSet::new();
    for (idx, item) in parsed.items.iter().enumerate() {
        match item {
            Item::Struct(s) if s.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Enum(e) if e.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Fn(f) if f.sig.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Trait(t) if t.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Impl(imp) if format_ty_name(&imp.self_ty) == entity_name => {
                indices.insert(idx);
            }
            Item::Mod(mod_item) => {
                if mod_item.attrs.iter().any(|a| a.path().is_ident("cfg")) {
                    if let Some((_brace, items)) = &mod_item.content {
                        for inner in items {
                            if let Item::Fn(test_fn) = inner {
                                if NameVisitor::new(entity_name).visit_fn(test_fn) {
                                    indices.insert(idx);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    indices
}
