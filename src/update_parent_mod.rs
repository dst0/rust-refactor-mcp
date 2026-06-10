use crate::extract::to_snake_case;
use std::path::PathBuf;
use syn::Item;
/// Backward compat: text-based parent mod update (kept for external callers).
pub fn update_parent_mod(target_folder: &str, entity_name: &str) {
    let mut module_file = PathBuf::from(target_folder).join("lib.rs");
    if !module_file.exists() {
        module_file = PathBuf::from(target_folder).join("mod.rs");
    }
    if !module_file.exists() {
        module_file = PathBuf::from(target_folder).join("main.rs");
    }
    let content = match std::fs::read_to_string(&module_file) {
        Ok(c) => c,
        Err(_) => return,
    };
    let Ok(parsed) = syn::parse_file(&content) else {
        return;
    };
    let mod_name = to_snake_case(entity_name);
    if mod_name.is_empty() || mod_name == "_" {
        return;
    }
    let mod_ident = if let Ok(id) = syn::parse_str::<syn::Ident>(&mod_name) {
        id
    } else if let Ok(id) = syn::parse_str::<syn::Ident>(&format!("r#{}", mod_name)) {
        id
    } else {
        return;
    };
    for item in &parsed.items {
        if let Item::Mod(m) = item {
            if m.ident == mod_ident {
                return;
            }
        }
    }
    let mut new_file = parsed;
    if let Ok(mod_item) = syn::parse2::<Item>(quote::quote!(pub mod #mod_ident;)) {
        new_file.items.insert(0, mod_item);
    }
    let new_content = prettyplease::unparse(&new_file);
    let _ = std::fs::write(&module_file, new_content);
}
