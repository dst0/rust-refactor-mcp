use crate::collect_use_names::collect_use_names;
use crate::extract_module_path::extract_module_path;
use crate::has_use_ref::has_use_ref;
use crate::qualpathreplacer::QualPathReplacer;
use crate::collect_referenced_identifiers::collect_referenced_identifiers;
use crate::is_import_used::is_import_used;
use std::path::PathBuf;
use proc_macro2::Span;
use syn::{Item, ItemUse};
pub fn update_usage_files(
    target_folder: &str,
    entity_name: &str,
    old_module_hint: Option<&str>,
) -> Result<Vec<String>, String> {
    let source_dir = PathBuf::from(target_folder);
    let mut updated = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&source_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension() != Some(std::ffi::OsStr::new("rs")) {
                continue;
            }
            if path.file_stem().map(|s| s.to_string_lossy())
                == Some(entity_name.to_lowercase().into())
            {
                continue;
            }
            let file_content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut parsed = match syn::parse_file(&file_content) {
                Ok(p) => p,
                Err(_) => continue,
            };
            let new_module = entity_name.to_lowercase();
            let mut changed = false;
            let mut old_mod = old_module_hint.map(|s| s.to_string());
            if old_mod.is_none() {
                for item in &parsed.items {
                    if let Item::Use(iu) = item {
                        if has_use_ref(entity_name, &iu.tree) {
                            let full_path = extract_module_path(&iu.tree, entity_name);
                            old_mod = full_path
                                .split("::")
                                .last()
                                .map(|s| s.to_string());
                            break;
                        }
                    }
                }
            }
            if let Some(om) = old_mod {
                let mut replacer = QualPathReplacer {
                    old_mod: om,
                    entity_name: entity_name.to_string(),
                    new_mod: new_module.clone(),
                    changed: false,
                };
                syn::visit_mut::VisitMut::visit_file_mut(&mut replacer, &mut parsed);
                if replacer.changed {
                    changed = true;
                }
            }
            let mut new_items: Vec<Item> = Vec::new();
            for item in parsed.items {
                match &item {
                    Item::Use(iu) if has_use_ref(entity_name, &iu.tree) => {
                        changed = true;
                        let names = collect_use_names(&iu.tree);
                        let prefix = extract_module_path(&iu.tree, entity_name);
                        for name in &names {
                            if name != entity_name {
                                let use_str = format!("use {}::{};", prefix, name);
                                if let Ok(parsed_use) = syn::parse_str::<
                                    ItemUse,
                                >(&use_str) {
                                    new_items.push(Item::Use(parsed_use));
                                }
                            }
                        }
                    }
                    _ => {
                        new_items.push(item);
                    }
                }
            }
            if changed {
                let new_mod_ident = syn::Ident::new(&new_module, Span::call_site());
                let entity_ident = syn::Ident::new(entity_name, Span::call_site());
                let new_use: Item = syn::parse2(
                        quote::quote!(use crate ::# new_mod_ident::# entity_ident;),
                    )
                    .unwrap();
                new_items.insert(0, new_use);
                let used = collect_referenced_identifiers(&new_items);
                new_items
                    .retain(|item| {
                        if let Item::Use(iu) = item {
                            let names = collect_use_names(&iu.tree);
                            is_import_used(&names, &used)
                        } else {
                            true
                        }
                    });
                let final_file = syn::File {
                    shebang: None,
                    attrs: Vec::new(),
                    items: new_items,
                };
                let new_content = prettyplease::unparse(&final_file);
                std::fs::write(&path, &new_content)
                    .map_err(|e| format!("Cannot update {}: {}", path.display(), e))?;
                updated.push(path.to_string_lossy().to_string());
            }
        }
    }
    Ok(updated)
}
