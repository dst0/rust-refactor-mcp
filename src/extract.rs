use crate::update_parent_mod::update_parent_mod;
use crate::merge_spans::merge_spans;
use crate::remove_spans::remove_spans;
use crate::namevisitor::NameVisitor;
use crate::identcollector::IdentCollector;
use crate::bytespan::ByteSpan;
use crate::extractresult::ExtractResult;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, Item, ItemFn, ItemUse, Type, UseTree};
pub fn extract_entity(
    source: &str,
    entity_name: &str,
    target_folder: &str,
    _entity_type: Option<&str>,
    source_file_path: Option<&str>,
) -> Result<ExtractResult, String> {
    let parsed = syn::parse_file(source).map_err(|e| format!("Parse error: {}", e))?;
    let extracted_indices = find_extracted_indices(&parsed, entity_name);
    if extracted_indices.is_empty() {
        return Err(format!("Entity '{}' not found", entity_name));
    }
    let mut remaining = Vec::new();
    let mut extracted: Vec<Item> = Vec::new();
    let mut test_items: Vec<ItemFn> = Vec::new();
    for (idx, item) in parsed.items.iter().enumerate() {
        if extracted_indices.contains(&idx) {
            match item {
                Item::Fn(f) if is_test_fn(f) => {
                    test_items.push(f.clone());
                }
                _ => {
                    extracted.push(item.clone());
                }
            }
        } else {
            remaining.push(item.clone());
        }
    }
    let mut all_spans: Vec<ByteSpan> = Vec::new();
    for item in &extracted {
        all_spans.push(span_to_byte(&item.span(), source));
    }
    for tfn in &test_items {
        all_spans.push(span_to_byte(&tfn.span(), source));
    }
    all_spans.sort_by_key(|s| s.start);
    let merged_spans = merge_spans(all_spans);
    let mut items_extracted = Vec::new();
    for item in &extracted {
        items_extracted.push(format!("{}: {}", item_type(item), entity_name));
    }
    for tfn in &test_items {
        items_extracted.push(format!("test: {}", tfn.sig.ident));
    }
    let new_module = entity_name.to_lowercase();
    let source_stem = source_file_path
        .as_ref()
        .and_then(|p| {
            PathBuf::from(p).file_stem().map(|s| s.to_string_lossy().to_string())
        });
    let same_file = source_stem.as_deref() == Some(&new_module);
    let new_path = PathBuf::from(target_folder).join(format!("{}.rs", new_module));
    let new_file_path = if same_file {
        source_file_path.map(|p| p.to_string()).unwrap_or_default()
    } else {
        new_path.to_string_lossy().to_string()
    };
    if !same_file {
        let needed_imports = detect_needed_imports_for_extracted(
            &parsed,
            &extracted,
            entity_name,
        );
        let mut new_file = File {
            shebang: None,
            attrs: Vec::new(),
            items: Vec::new(),
        };
        for imp in &needed_imports {
            new_file.items.push(Item::Use(imp.clone()));
        }
        for item in &extracted {
            new_file.items.push(item.clone());
        }
        let cross_refs = detect_cross_refs_for_extracted(
            &parsed,
            &extracted,
            entity_name,
            source_file_path,
        );
        for imp in cross_refs {
            new_file.items.push(Item::Use(imp));
        }
        let filename = format!("{}.rs", new_module);
        let new_path = PathBuf::from(target_folder).join(&filename);
        fs::create_dir_all(PathBuf::from(target_folder))
            .map_err(|e| format!("Cannot create dir: {}", e))?;
        let content = prettyplease::unparse(&new_file);
        fs::write(&new_path, content).map_err(|e| format!("Cannot write file: {}", e))?;
    }
    let test_file_path = if !test_items.is_empty() {
        let mut test_content = String::from("#[cfg(test)]\nmod tests {\n");
        for tfn in &test_items {
            let fn_file = File {
                shebang: None,
                attrs: Vec::new(),
                items: vec![Item::Fn(tfn.clone())],
            };
            test_content.push_str(&prettyplease::unparse(&fn_file));
            test_content.push_str("\n\n");
        }
        test_content.push_str("}\n");
        let test_filename = format!("{}_tests.rs", entity_name.to_lowercase());
        let test_path = PathBuf::from(target_folder).join(&test_filename);
        fs::write(&test_path, &test_content)
            .map_err(|e| format!("Cannot write test file: {}", e))?;
        Some(test_path.to_string_lossy().to_string())
    } else {
        None
    };
    if !same_file {
        let used_ids = collect_referenced_identifiers(&remaining);
        if used_ids.contains(entity_name) {
            let new_mod_ident = syn::Ident::new(&new_module, Span::call_site());
            let entity_ident = syn::Ident::new(entity_name, Span::call_site());
            let source_use: Item = syn::parse2(
                    quote::quote!(use crate ::# new_mod_ident::# entity_ident;),
                )
                .unwrap();
            remaining.insert(0, source_use);
        }
        let remaining_file = File {
            shebang: None,
            attrs: Vec::new(),
            items: remaining,
        };
        let cleaned = cleanup_imports_in_ast(&remaining_file, &used_ids);
        let updated_content = prettyplease::unparse(&cleaned);
        fs::write(source_file_path.unwrap_or("source.rs"), &updated_content)
            .map_err(|e| format!("Cannot write updated source: {}", e))?;
        let usage_updated = update_usage_files(target_folder, entity_name)?;
        update_parent_mod(target_folder, entity_name);
        Ok(ExtractResult {
            new_file_path,
            test_file_path,
            items_extracted,
            usage_files_updated: usage_updated,
            extracted_spans: merged_spans,
        })
    } else {
        Ok(ExtractResult {
            new_file_path: source_file_path.map(|p| p.to_string()).unwrap_or_default(),
            test_file_path,
            items_extracted,
            usage_files_updated: Vec::new(),
            extracted_spans: merged_spans,
        })
    }
}
/// Backward compat: text-based span removal (kept for external callers).
pub fn remove_from_source(source: &str, spans: &[ByteSpan]) -> Result<String, String> {
    Ok(remove_spans(source, spans))
}
/// Backward compat: text-based import check (kept for external callers).
pub fn ensure_source_import(
    source: &str,
    entity_name: &str,
    new_file_path: &str,
) -> String {
    let Ok(parsed) = syn::parse_file(source) else {
        return source.to_string();
    };
    let used = collect_referenced_identifiers(&parsed.items);
    if !used.contains(entity_name) {
        return source.to_string();
    }
    for item in &parsed.items {
        if let Item::Use(iu) = item {
            if has_use_ref(entity_name, &iu.tree) {
                return source.to_string();
            }
        }
    }
    let mod_name = PathBuf::from(new_file_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(entity_name)
        .to_lowercase();
    let mod_ident = syn::Ident::new(&mod_name, Span::call_site());
    let entity_ident = syn::Ident::new(entity_name, Span::call_site());
    let mut new_file = parsed.clone();
    let use_stmt: Item = syn::parse2(
            quote::quote!(use crate ::# mod_ident::# entity_ident;),
        )
        .unwrap();
    new_file.items.insert(0, use_stmt);
    prettyplease::unparse(&new_file)
}
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
pub fn is_test_fn(f: &ItemFn) -> bool {
    f.attrs.iter().any(|a| a.path().is_ident("cfg"))
}
pub fn collect_referenced_identifiers(items: &[Item]) -> HashSet<String> {
    let mut visitor = IdentCollector {
        found: HashSet::new(),
    };
    for item in items {
        if matches!(item, Item::Use(_)) {
            continue;
        }
        visitor.visit_item(item);
    }
    visitor.found
}
pub fn is_import_used(names: &[String], used_ids: &HashSet<String>) -> bool {
    for name in names {
        if used_ids.contains(name) {
            return true;
        }
        if name == "Spanned" && used_ids.contains("span") {
            return true;
        }
    }
    false
}
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
            if names.iter().any(|n| used.contains(n.as_str()) && n != _entity_name) {
                needed.push(iu.clone());
            }
        }
    }
    needed
}
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
pub fn update_usage_files(
    target_folder: &str,
    entity_name: &str,
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
            let Ok(parsed) = syn::parse_file(&file_content) else {
                continue;
            };
            let new_module = entity_name.to_lowercase();
            let mut changed = false;
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
pub fn format_ty_name(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => {
            tp.path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_else(|| {
                    tp.path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default()
                })
        }
        _ => format!("{:?}", ty),
    }
}
pub fn item_type(item: &Item) -> &'static str {
    match item {
        Item::Struct(_) => "struct",
        Item::Enum(_) => "enum",
        Item::Fn(_) => "fn",
        Item::Trait(_) => "trait",
        Item::Impl(_) => "impl",
        _ => "item",
    }
}
pub fn span_to_byte(span: &proc_macro2::Span, source: &str) -> ByteSpan {
    let start = span.start();
    let end = span.end();
    ByteSpan::new(
        line_col_to_byte(source, start.line, start.column),
        line_col_to_byte(source, end.line, end.column),
    )
}
pub fn line_col_to_byte(source: &str, line: usize, column: usize) -> usize {
    let target_line = line.saturating_sub(1);
    let mut byte_offset = 0;
    let mut current_line: usize = 0;
    let mut current_col: usize = 0;
    for c in source.chars() {
        if current_line == target_line && current_col == column {
            return byte_offset;
        }
        byte_offset += c.len_utf8();
        if c == '\n' {
            current_line += 1;
            current_col = 0;
        } else {
            current_col += 1;
        }
    }
    byte_offset
}
pub fn has_use_ref(entity_name: &str, tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(p) => p.ident == entity_name || has_use_ref(entity_name, &p.tree),
        UseTree::Name(n) => n.ident == entity_name,
        UseTree::Rename(r) => r.ident == entity_name,
        UseTree::Group(g) => g.items.iter().any(|t| has_use_ref(entity_name, t)),
        UseTree::Glob(_) => false,
    }
}
pub fn extract_module_path(tree: &UseTree, entity_name: &str) -> String {
    match tree {
        UseTree::Path(p) => {
            if p.ident == entity_name {
                String::new()
            } else {
                let rest = extract_module_path(&p.tree, entity_name);
                if rest.is_empty() {
                    p.ident.to_string()
                } else {
                    format!("{}::{}", p.ident, rest)
                }
            }
        }
        UseTree::Group(g) => {
            for item in &g.items {
                let path = extract_module_path(item, entity_name);
                if !path.is_empty() {
                    return path;
                }
            }
            String::new()
        }
        _ => String::new(),
    }
}
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
