use crate::identcollector::IdentCollector;
use crate::merge_spans::merge_spans;
use crate::update_parent_mod::update_parent_mod;
use proc_macro2::Span;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, Item, ItemFn, ItemUse, Type, UseTree};
pub fn compute_module_name(entity_name: &str, items: &[Item]) -> String {
    let mut new_module = to_snake_case(entity_name);
    let is_type = items.iter().any(|item| {
        matches!(
            item,
            Item::Struct(_) | Item::Enum(_) | Item::Type(_) | Item::Trait(_)
        )
    });
    let is_fn = items.iter().any(|item| matches!(item, Item::Fn(_)));
    let is_macro = items.iter().any(|item| matches!(item, Item::Macro(_)));
    let is_const = items.iter().any(|item| matches!(item, Item::Const(_) | Item::Static(_)));

    if new_module == entity_name {
        if is_type {
            new_module = format!("{}_mod", new_module);
        } else if is_fn {
            new_module = format!("{}_impl", new_module);
        } else if is_macro {
            new_module = format!("{}_macro", new_module);
        } else if is_const {
            new_module = format!("{}_const", new_module);
        }
    }
    new_module
}

pub fn extract_entity(
    source: &str,
    entity_name: &str,
    target_folder: &str,
    entity_types: Option<Vec<String>>,
    source_file_path: Option<&str>,
    cached_files: Option<&Vec<PathBuf>>,
    generate_reexport: bool,
    fix_vis: Option<&str>,
    fix_macros: Option<&str>,
) -> Result<ExtractResult, String> {
    let parsed = syn::parse_file(source).map_err(|e| format!("Parse error: {}", e))?;
    let extracted_indices = find_extracted_indices(&parsed, entity_name, entity_types.as_deref());
    if extracted_indices.is_empty() {
        return Err(format!(
            "Entity '{}' of types {:?} not found",
            entity_name, entity_types
        ));
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

    let mut extracted_private_fields = Vec::new();
    let mut extracted_macros = Vec::new();
    for item in &extracted {
        if let Item::Struct(s) = item {
            for field in &s.fields {
                if let syn::Visibility::Inherited = field.vis {
                    if let Some(ident) = &field.ident {
                        extracted_private_fields.push(ident.to_string());
                    }
                }
            }
        } else if let Item::Macro(m) = item {
            if let Some(ident) = &m.ident {
                extracted_macros.push(ident.to_string());
            }
        }
    }
    
    let target_fields: Vec<&str> = extracted_private_fields.iter().map(|s| s.as_str()).collect();
    let target_macros: Vec<&str> = extracted_macros.iter().map(|s| s.as_str()).collect();
    let mut field_visitor = crate::usage_analysis::FieldUsageVisitor::new(target_fields);
    let mut macro_visitor = crate::usage_analysis::MacroUsageVisitor::new(target_macros);

    for item in &remaining {
        syn::visit::Visit::visit_item(&mut field_visitor, item);
        syn::visit::Visit::visit_item(&mut macro_visitor, item);
    }

    let mut issues = Vec::new();
    if !field_visitor.used_fields.is_empty() && fix_vis != Some("pub_crate") {
        issues.push(format!("Private fields {:?} are used in the remaining code. Pass --fix-vis=pub_crate to auto-fix.", field_visitor.used_fields));
    }
    if !macro_visitor.used_macros.is_empty() && fix_macros != Some("promote") {
        issues.push(format!("Macros {:?} are used in the remaining code. Pass --fix-macros=promote to auto-fix.", macro_visitor.used_macros));
    }
    if !issues.is_empty() {
        return Err(format!("Extraction aborted due to code breakage risks:\n{}", issues.join("\n")));
    }

    if fix_vis == Some("pub_crate") && !field_visitor.used_fields.is_empty() {
        for item in &mut extracted {
            if let Item::Struct(s) = item {
                for field in &mut s.fields {
                    if let syn::Visibility::Inherited = field.vis {
                        if let Some(ident) = &field.ident {
                            if field_visitor.used_fields.contains(&ident.to_string()) {
                                field.vis = syn::parse_quote!(pub(crate));
                            }
                        }
                    }
                }
            }
        }
    }

    let mut macro_promotions = Vec::new();
    if fix_macros == Some("promote") && !macro_visitor.used_macros.is_empty() {
        for mac_name in &macro_visitor.used_macros {
            let ident = syn::Ident::new(mac_name, proc_macro2::Span::call_site());
            let pub_use: Item = syn::parse_quote!(pub(crate) use #ident;);
            macro_promotions.push(pub_use);
            
            let new_module = compute_module_name(entity_name, &extracted);
            let mod_ident = syn::Ident::new(&new_module, proc_macro2::Span::call_site());
            let use_import: Item = syn::parse_quote!(use crate::#mod_ident::#ident;);
            remaining.insert(0, use_import);
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
    let mut new_module = compute_module_name(entity_name, &extracted);
    // Check for naming collisions with existing imports
    for item in &parsed.items {
        if let Item::Use(iu) = item {
            let names = collect_use_names(&iu.tree);
            if names.contains(&new_module) {
                new_module = format!("{}_mod", new_module);
                break;
            }
        }
    }
    let source_stem = source_file_path.as_ref().and_then(|p| {
        PathBuf::from(p)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
    });
    let same_file = source_stem.as_deref() == Some(&new_module);
    let file_path = PathBuf::from(source_file_path.unwrap_or(""));
    let parent_dir = file_path.parent().unwrap_or(Path::new(""));
    let new_path = parent_dir.join(format!("{}.rs", new_module));
    let new_file_path = if same_file {
        source_file_path.map(|p| p.to_string()).unwrap_or_default()
    } else {
        new_path.to_string_lossy().to_string()
    };
    if !same_file {
        let needed_imports = detect_needed_imports_for_extracted(&parsed, &extracted, entity_name);
        let mut new_file = if new_path.exists() {
            let content = std::fs::read_to_string(&new_path).unwrap_or_default();
            syn::parse_file(&content).unwrap_or_else(|_| File {
                shebang: parsed.shebang.clone(),
                attrs: parsed.attrs.clone(),
                items: Vec::new(),
            })
        } else {
            File {
                shebang: parsed.shebang.clone(),
                attrs: parsed.attrs.clone(),
                items: Vec::new(),
            }
        };

        let mut already_exists = false;
        for item in &new_file.items {
            if get_item_name(item).as_deref() == Some(entity_name) {
                already_exists = true;
                break;
            }
        }

        if !already_exists {
            // Collect names already defined in new_file (e.g. from a prior extraction pass)
            let already_defined: HashSet<String> = new_file.items.iter().filter_map(|item| get_item_name(item)).collect();

            for imp in &needed_imports {
                // Skip imports whose name conflicts with existing definitions
                let imp_names = collect_use_names(&imp.tree);
                if imp_names.iter().any(|n| already_defined.contains(n)) {
                    continue;
                }
                let imp_str = quote::quote!(#imp).to_string();
                let is_dup = new_file.items.iter().any(|existing| {
                    if let Item::Use(existing_imp) = existing {
                        quote::quote!(#existing_imp).to_string() == imp_str
                    } else {
                        false
                    }
                });
                if !is_dup {
                    new_file.items.push(Item::Use(imp.clone()));
                }
            }
            for item in &extracted {
                let mut item = item.clone();
                make_item_pub(&mut item);
                new_file.items.push(item);
            }
            for promo in macro_promotions {
                new_file.items.push(promo);
            }
            let cross_refs = detect_cross_refs_for_extracted(&parsed, &extracted, entity_name, source_file_path);
            for imp in cross_refs {
                let imp_str = quote::quote!(#imp).to_string();
                let is_dup = new_file.items.iter().any(|existing| {
                    if let Item::Use(existing_imp) = existing {
                        quote::quote!(#existing_imp).to_string() == imp_str
                    } else {
                        false
                    }
                });
                if !is_dup {
                    new_file.items.push(Item::Use(imp));
                }
            }
        }

        // Clean up unused imports in the newly extracted file
        let used_ids_new = collect_referenced_identifiers(&new_file.items);
        let cleaned_new_file = cleanup_imports_in_ast(&new_file, &used_ids_new);

        let filename = format!("{}.rs", new_module);
        let new_path = PathBuf::from(target_folder).join(&filename);
        fs::create_dir_all(PathBuf::from(target_folder))
            .map_err(|e| format!("Cannot create dir: {}", e))?;
        let content = prettyplease::unparse(&cleaned_new_file);
        fs::write(&new_path, content).map_err(|e| format!("Cannot write file: {}", e))?;
        let _ = std::process::Command::new("rustfmt")
            .arg("--edition")
            .arg("2024")
            .arg(&new_path)
            .status();
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
        let test_filename = format!("{}_tests.rs", to_snake_case(entity_name));
        let test_path = PathBuf::from(target_folder).join(&test_filename);
        fs::write(&test_path, &test_content)
            .map_err(|e| format!("Cannot write test file: {}", e))?;
        let _ = std::process::Command::new("rustfmt")
            .arg("--edition")
            .arg("2024")
            .arg(&test_path)
            .status();
        Some(test_path.to_string_lossy().to_string())
    } else {
        None
    };
    if !same_file {
        let used_by_extracted = collect_referenced_identifiers(&extracted);
        for item in &mut remaining {
            if let Some(name) = get_item_name(item) {
                if used_by_extracted.contains(&name) {
                    make_item_pub(item);
                }
            }
        }

        let used_ids = collect_referenced_identifiers(&remaining);
        let mut is_pub = false;
        for idx in &extracted_indices {
            if let Some(item) = parsed.items.get(*idx) {
                if is_item_pub(item) {
                    is_pub = true;
                    break;
                }
            }
        }

        if generate_reexport && (used_ids.contains(entity_name) || is_pub) {
            let full_mod_path = format!("crate::{}", new_module);
            let vis_prefix = if is_pub { "pub " } else { "pub(crate) " };
            let mod_ident = syn::Ident::new(&new_module, proc_macro2::Span::call_site());
            let mod_path = format!("{}.rs", new_module);
            let vis: syn::Visibility = if is_pub { syn::parse_quote!(pub) } else { syn::parse_quote!(pub(crate)) };
            let mod_use: Item = syn::parse_quote!(#[path = #mod_path] #vis mod #mod_ident;);
            remaining.insert(0, mod_use);
            
            let escaped_entity = escape_path_segment(entity_name);
            let use_str = format!("{}use {}::{};", vis_prefix, full_mod_path, escaped_entity);
            if let Ok(mut source_use) = syn::parse_str::<Item>(&use_str) {
                if let Item::Use(ref mut iu) = source_use {
                    if let Some(first_extracted) = extracted.first() {
                        iu.attrs = get_item_attrs(first_extracted).unwrap_or_default();
                    }
                }
                remaining.insert(1, source_use);
            }
        }
        let remaining_file = File {
            shebang: parsed.shebang.clone(),
            attrs: parsed.attrs.clone(),
            items: remaining,
        };
        let cleaned = cleanup_imports_in_ast(&remaining_file, &used_ids);
        let updated_content = prettyplease::unparse(&cleaned);
        fs::write(source_file_path.unwrap_or("source.rs"), &updated_content)
            .map_err(|e| format!("Cannot write updated source: {}", e))?;
        let _ = std::process::Command::new("rustfmt")
            .arg("--edition")
            .arg("2024")
            .arg(source_file_path.unwrap_or("source.rs"))
            .status();

        let usage_updated = update_usage_files(
            target_folder,
            entity_name,
            &new_module,
            source_stem.as_deref(),
            source_file_path,
            cached_files,
        )?;
        update_parent_mod(&target_folder, &new_module);
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

pub fn find_extracted_indices(
    parsed: &File,
    entity_name: &str,
    entity_types: Option<&[String]>,
) -> HashSet<usize> {
    let mut indices = HashSet::new();
    for (idx, item) in parsed.items.iter().enumerate() {
        if let Some(et) = entity_types {
            if !et.contains(&item_type(item).to_string()) {
                continue;
            }
        }
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
            Item::Type(t) if t.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Const(c) if c.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Static(s) if s.ident == entity_name => {
                indices.insert(idx);
            }
            Item::Impl(imp) => {
                let self_ty_name = format_ty_name(&imp.self_ty);
                if self_ty_name == entity_name {
                    indices.insert(idx);
                } else if let Some((_, tr_path, _)) = &imp.trait_ {
                    let tr_str = quote::quote!(# tr_path).to_string();
                    if tr_str.contains(entity_name) {
                        indices.insert(idx);
                    }
                }
            }
            _ => {}
        }
    }
    indices
}
pub fn is_test_fn(f: &ItemFn) -> bool {
    f.attrs.iter().any(|a| {
        a.path().is_ident("test") || 
        (a.path().is_ident("cfg") && quote::quote!(#a).to_string().contains("test"))
    })
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
        // Special case for traits whose methods are used but trait name is not explicitly mentioned
        if name == "Spanned" && used_ids.contains("span") {
            return true;
        }
        if name == "Visit" {
            return true;
        }
        if name == "Deserialize" || name == "Serialize" {
            return true;
        }
    }
    false
}
pub fn cleanup_imports_in_ast(parsed: &File, used_ids: &HashSet<String>) -> File {
    let mut cleaned = parsed.clone();
    cleaned.items.retain(|item| {
        if let Item::Use(iu) = item {
            if !matches!(iu.vis, syn::Visibility::Inherited) {
                return true;
            }
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
    _extracted: &[Item],
    entity_name: &str,
) -> Vec<ItemUse> {
    parsed
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Use(iu) = item {
                // Skip re-exports that reference the entity itself (added by prior extraction passes)
                let names = collect_use_names(&iu.tree);
                if names.iter().any(|n| n == entity_name) {
                    return None;
                }
                let mut new_iu = iu.clone();
                transform_self_to_crate(&mut new_iu.tree);
                Some(new_iu)
            } else {
                None
            }
        })
        .collect()
}

pub fn transform_self_to_crate(root_tree: &mut UseTree) {
    let mut stack = vec![(root_tree, true)];
    while let Some((tree, is_leading)) = stack.pop() {
        match tree {
            UseTree::Path(p) => {
                if is_leading && p.ident == "self" {
                    p.ident = syn::Ident::new("super", p.ident.span());
                    stack.push((&mut p.tree, false));
                } else if is_leading && p.ident == "super" {
                    let old_tree = p.tree.clone();
                    *p.tree = UseTree::Path(syn::UsePath {
                        ident: syn::Ident::new("super", Span::call_site()),
                        colon2_token: syn::token::PathSep::default(),
                        tree: old_tree,
                    });
                    if let UseTree::Path(ref mut inner_p) = *p.tree {
                        stack.push((&mut inner_p.tree, false));
                    }
                } else {
                    stack.push((&mut p.tree, false));
                }
            }
            UseTree::Group(g) => {
                for item in &mut g.items {
                    stack.push((item, is_leading));
                }
            }
            _ => {}
        }
    }
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
            Item::Type(t) if t.ident != entity_name => Some(t.ident.to_string()),
            Item::Const(c) if c.ident != entity_name => Some(c.ident.to_string()),
            Item::Static(s) if s.ident != entity_name => Some(s.ident.to_string()),
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
    let mut module_path = "crate".to_string();
    if let Some(path) = source_file_path {
        let p = std::path::PathBuf::from(path);
        let mut parts = Vec::new();
        for c in p.components() {
            if let std::path::Component::Normal(n) = c {
                let s = n.to_str().unwrap();
                if s != "src" {
                    if s.ends_with(".rs") {
                        let stem = s.trim_end_matches(".rs");
                        if stem != "mod" && stem != "lib" && stem != "main" {
                            parts.push(escape_path_segment(stem));
                        }
                    } else {
                        parts.push(escape_path_segment(s));
                    }
                }
            }
        }
        if !parts.is_empty() {
            module_path = format!("crate::{}", parts.join("::"));
        }
    } else {
        module_path = "super".to_string();
    }
    let escaped_names: Vec<String> = needed.iter().map(|n| escape_path_segment(n)).collect();
    let names = escaped_names.join(", ");
    let use_str = format!("use {}::{{{}}};", module_path, names);
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
    new_module_name: &str,
    old_module_hint: Option<&str>,
    exclude_path: Option<&str>,
    cached_files: Option<&Vec<PathBuf>>,
) -> Result<Vec<String>, String> {
    let mut updated = Vec::new();
    let exclude_canonical = exclude_path.and_then(|p| fs::canonicalize(p).ok());
    let mut files_to_process = Vec::new();
    if let Some(cached) = cached_files {
        files_to_process = cached.clone();
    } else {
        collect_rs_files(PathBuf::from(target_folder), &mut files_to_process);
    }

    let total_files = files_to_process.len();
    for (i, path) in files_to_process.into_iter().enumerate() {
        use std::io::Write;
        print!(
            "\r    Usage scan: {}/{} files (processing: {})             ",
            i,
            total_files,
            path.file_name().unwrap_or_default().to_string_lossy()
        );
        std::io::stdout().flush().ok();

        if let Some(ref ex) = exclude_canonical {
            if let Ok(p_can) = fs::canonicalize(&path) {
                if p_can == *ex {
                    continue;
                }
            }
        }
        if path.file_stem().map(|s| s.to_string_lossy()) == Some(to_snake_case(entity_name).into())
        {
            continue;
        }
        let file_content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Fast check before parsing
        if !file_content.contains(entity_name) {
            continue;
        }

        let parsed = match syn::parse_file(&file_content) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let new_module = new_module_name.to_string();
        let mut changed = false;

        // Determine the old module name for matching import paths
        let mut old_mod = old_module_hint.map(|s| s.to_string());
        if old_mod.is_none() {
            for item in &parsed.items {
                if let Item::Use(iu) = item {
                    if has_use_ref(entity_name, &iu.tree) {
                        let full_path = extract_module_path(&iu.tree, entity_name);
                        old_mod = full_path.split("::").last().map(|s| s.to_string());
                        break;
                    }
                }
            }
        }
        // Note: We deliberately do NOT run QualPathReplacer here.
        // Since re-exports are always generated, old qualified paths (e.g. ping::channel())
        // still resolve correctly via the re-export and should not be rewritten.

        let mut new_items: Vec<Item> = Vec::new();
        let mut extracted_attrs: Vec<syn::Attribute> = Vec::new();
        for item in parsed.items {
            match &item {
                Item::Use(iu) if has_use_ref(entity_name, &iu.tree) => {
                    if !iu.attrs.is_empty() {
                        extracted_attrs = iu.attrs
                            .iter()
                            .filter(|a| a.path().is_ident("cfg"))
                            .cloned()
                            .collect();
                    }
                    let prefix = extract_module_path(&iu.tree, entity_name);
                    let prefix_last = prefix.split("::").last().unwrap_or("");
                    let matches_old_mod = if let Some(ref om) = old_mod {
                        let is_crate_root = om == "lib" || om == "main";
                        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                        let parent_dir = path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()).unwrap_or("");
                        let is_same_mod = file_stem == *om || parent_dir == *om;
                        let prefix_first = prefix.split("::").next().unwrap_or("");
                        // Only match crate-internal imports (crate::, self::, super::, or relative)
                        let is_crate_internal = prefix_first.is_empty()
                            || prefix_first == "crate"
                            || prefix_first == "self"
                            || prefix_first == "super"
                            || prefix_first == *om;
                        
                        is_crate_internal && (
                            (prefix_last.is_empty() && is_same_mod)
                            || (prefix_last == "super" && is_same_mod)
                            || (prefix_last == "self" && is_same_mod)
                            || prefix_last == *om 
                            || (prefix_last == "crate" && is_crate_root)
                        )
                    } else {
                        true
                    };
                    if !matches_old_mod {
                        new_items.push(item);
                        continue;
                    }
                    changed = true;
                    let vis = &iu.vis;
                    let vis_prefix = if matches!(vis, syn::Visibility::Inherited) {
                        String::new()
                    } else {
                        format!("{} ", quote::quote!(#vis))
                    };
                    fn get_all_paths(tree: &syn::UseTree, mut current: Vec<String>) -> Vec<Vec<String>> {
                        match tree {
                            syn::UseTree::Path(p) => {
                                current.push(p.ident.to_string());
                                get_all_paths(&p.tree, current)
                            }
                            syn::UseTree::Name(n) => {
                                current.push(n.ident.to_string());
                                vec![current]
                            }
                            syn::UseTree::Rename(r) => {
                                current.push(r.ident.to_string());
                                current.push(format!("as {}", r.rename));
                                vec![current]
                            }
                            syn::UseTree::Group(g) => {
                                let mut paths = Vec::new();
                                for item in &g.items {
                                    paths.extend(get_all_paths(item, current.clone()));
                                }
                                paths
                            }
                            syn::UseTree::Glob(_) => {
                                current.push("*".to_string());
                                vec![current]
                            }
                        }
                    }

                    let paths = get_all_paths(&iu.tree, Vec::new());
                    for path in paths {
                        // Find the leaf position: last segment that isn't "as ..." or "*"
                        let leaf_idx = path.iter().rposition(|s| !s.starts_with("as ") && s != "*");
                        // Check if the leaf segment matches entity_name
                        if let Some(idx) = leaf_idx.filter(|&i| path[i] == entity_name) {
                            // It's the entity or something inside it (like an enum variant)!
                            // Path should be full_mod_path::entity_name::[rest...]
                            let full_mod_path = format!("crate::{}", new_module);
                            let mut new_path_str = format!("{}::{}", full_mod_path, escape_path_segment(entity_name));
                            for segment in &path[idx + 1..] {
                                if segment.starts_with("as ") {
                                    new_path_str.push_str(&format!(" {}", segment));
                                } else if segment == "*" {
                                    new_path_str.push_str("::*");
                                } else {
                                    new_path_str.push_str(&format!("::{}", escape_path_segment(segment)));
                                }
                            }
                            let use_str = format!("{}use {};", vis_prefix, new_path_str);
                            if let Ok(mut parsed_use) = syn::parse_str::<ItemUse>(&use_str) {
                                parsed_use.attrs = extracted_attrs.clone();
                                new_items.push(Item::Use(parsed_use));
                            } else {
                                println!("WARNING: Failed to parse generated use (idx block): {}", use_str);
                            }
                        } else {
                            // Unrelated item in the same group import. Keep it as is!
                            // Skip `self` imports (use super::self is invalid outside braces)
                            let last_meaningful = path.iter().filter(|s| !s.starts_with("as ")).last().map(|s| s.as_str());
                            if last_meaningful == Some("self") {
                                continue;
                            }
                            let mut new_path_str = String::new();
                            for (i, segment) in path.iter().enumerate() {
                                if segment.starts_with("as ") {
                                    new_path_str.push_str(&format!(" {}", segment));
                                } else if segment == "*" {
                                    new_path_str.push_str("*");
                                } else {
                                    new_path_str.push_str(&escape_path_segment(segment));
                                    if i < path.len() - 1 && !path[i+1].starts_with("as ") {
                                        new_path_str.push_str("::");
                                    }
                                }
                            }
                            let use_str = format!("{}use {};", vis_prefix, new_path_str);
                            if let Ok(mut parsed_use) = syn::parse_str::<ItemUse>(&use_str) {
                                parsed_use.attrs = extracted_attrs.clone();
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
            // NOTE: We deliberately skip the import cleanup here.
            // The cleanup_imports_in_ast call in extract_entity handles the extracted file.
            // Aggressive cleanup here can remove imports that are still needed by code
            // that was already extracted to other files (e.g., `use crate::client::dispatch`
            // removed after extracting SendRequest, then TrySendError extraction cleans it up).
            let final_file = syn::File {
                shebang: parsed.shebang.clone(),
                attrs: parsed.attrs.clone(),
                items: new_items,
            };
            let new_content = prettyplease::unparse(&final_file);
            std::fs::write(&path, &new_content)
                .map_err(|e| format!("Cannot update {}: {}", path.display(), e))?;
            updated.push(path.to_string_lossy().to_string());
        }
    }
    if total_files > 10 {
        println!("\r    Usage scan: done ({} files searched)                                                    ", total_files);
    }
    Ok(updated)
}

fn collect_rs_files(dir: PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|s| s.to_str()) == Some("target") {
                    continue;
                }
                collect_rs_files(path, files);
            } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                files.push(path);
            }
        }
    }
}
pub fn format_ty_name(ty: &Type) -> String {
    match ty {
        Type::Path(tp) => tp
            .path
            .get_ident()
            .map(|i| i.to_string())
            .unwrap_or_else(|| {
                tp.path
                    .segments
                    .last()
                    .map(|s| s.ident.to_string())
                    .unwrap_or_default()
            }),
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
        Item::Const(_) => "const",
        Item::Static(_) => "static",
        Item::Type(_) => "type",
        Item::Macro(_) => "macro",
        Item::Mod(_) => "mod",
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

pub fn make_item_pub(item: &mut Item) {
    let pub_vis = syn::Visibility::Public(syn::token::Pub::default());
    let promote_vis = |vis: &mut syn::Visibility| {
        if matches!(vis, syn::Visibility::Inherited) || matches!(vis, syn::Visibility::Restricted(_)) {
            *vis = pub_vis.clone();
        }
    };

    match item {
        Item::Fn(f) => promote_vis(&mut f.vis),
        Item::Struct(s) => {
            promote_vis(&mut s.vis);
            match &mut s.fields {
                syn::Fields::Named(fields) => {
                    for field in &mut fields.named {
                        promote_vis(&mut field.vis);
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &mut fields.unnamed {
                        promote_vis(&mut field.vis);
                    }
                }
                syn::Fields::Unit => {}
            }
        }
        Item::Enum(e) => {
            promote_vis(&mut e.vis);
            // Enum variants and their fields inherit the enum's visibility
            // We cannot put `pub` on them!
        }
        Item::Trait(t) => promote_vis(&mut t.vis),
        Item::Type(t) => promote_vis(&mut t.vis),
        Item::Const(c) => promote_vis(&mut c.vis),
        Item::Static(s) => promote_vis(&mut s.vis),
        Item::Mod(m) => promote_vis(&mut m.vis),
        _ => {}
    }
}

pub fn is_item_pub(item: &Item) -> bool {
    matches!(get_item_vis(item), Some(syn::Visibility::Public(_)))
}

pub fn get_item_vis(item: &Item) -> Option<syn::Visibility> {
    match item {
        Item::Fn(f) => Some(f.vis.clone()),
        Item::Struct(s) => Some(s.vis.clone()),
        Item::Enum(e) => Some(e.vis.clone()),
        Item::Trait(t) => Some(t.vis.clone()),
        Item::Type(t) => Some(t.vis.clone()),
        Item::Const(c) => Some(c.vis.clone()),
        Item::Static(s) => Some(s.vis.clone()),
        Item::Mod(m) => Some(m.vis.clone()),
        _ => None,
    }
}

pub fn get_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Struct(s) => Some(s.ident.to_string()),
        Item::Enum(e) => Some(e.ident.to_string()),
        Item::Fn(f) => Some(f.sig.ident.to_string()),
        Item::Trait(t) => Some(t.ident.to_string()),
        Item::Const(c) => Some(c.ident.to_string()),
        Item::Static(s) => Some(s.ident.to_string()),
        Item::Type(t) => Some(t.ident.to_string()),
        Item::Mod(m) => Some(m.ident.to_string()),
        Item::Macro(m) => m.ident.as_ref().map(|id| id.to_string()),
        _ => None,
    }
}

pub fn get_item_attrs(item: &Item) -> Option<Vec<syn::Attribute>> {
    let attrs = match item {
        Item::Struct(s) => Some(s.attrs.clone()),
        Item::Enum(e) => Some(e.attrs.clone()),
        Item::Fn(f) => Some(f.attrs.clone()),
        Item::Trait(t) => Some(t.attrs.clone()),
        Item::Const(c) => Some(c.attrs.clone()),
        Item::Static(s) => Some(s.attrs.clone()),
        Item::Type(t) => Some(t.attrs.clone()),
        Item::Mod(m) => Some(m.attrs.clone()),
        Item::Macro(m) => Some(m.attrs.clone()),
        Item::Use(u) => Some(u.attrs.clone()),
        _ => None,
    };
    attrs.map(|v| v.into_iter().filter(|a| a.path().is_ident("cfg")).collect())
}

use syn::visit_mut::{self, VisitMut};
pub struct QualPathReplacer {
    pub old_mod: String,
    pub entity_name: String,
    pub new_mod: String,
    pub changed: bool,
}
impl VisitMut for QualPathReplacer {
    fn visit_path_mut(&mut self, i: &mut syn::Path) {
        let len = i.segments.len();
        if len >= 2
            && i.segments[len - 2].ident == self.old_mod
            && i.segments[len - 1].ident == self.entity_name
        {
            if !self.new_mod.contains("::") {
                let new_segment = syn::PathSegment {
                    ident: syn::Ident::new(&self.new_mod, i.segments[len - 2].ident.span()),
                    arguments: syn::PathArguments::None,
                };
                i.segments.insert(len - 1, new_segment);
                self.changed = true;
            }
        }
        visit_mut::visit_path_mut(self, i);
    }
}

pub fn to_snake_case(s: &str) -> String {
    if s.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        return s.to_lowercase();
    }
    let mut snake = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
    }
    snake
}

pub fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

pub fn escape_path_segment(segment: &str) -> String {
    let keywords = [
        "type", "struct", "enum", "fn", "trait", "impl", "const", "static", "match", "let", "mut", "ref", "pub",
    ];
    if keywords.contains(&segment) {
        format!("r#{}", segment)
    } else {
        segment.to_string()
    }
}

pub fn get_full_module_path(target_folder: &str, new_module: &str) -> String {
    let mut path = PathBuf::from(target_folder);
    let mut components = Vec::new();
    components.push(new_module.to_string());

    while let Some(parent) = path.parent() {
        if path.file_name().and_then(|s| s.to_str()) == Some("src") {
            break;
        }
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            components.push(name.to_string());
        }
        path = parent.to_path_buf();
    }
    components.reverse();
    let escaped_components: Vec<String> = components
        .into_iter()
        .map(|c| escape_path_segment(&c))
        .collect();
    format!("crate::{}", escaped_components.join("::"))
}

use crate::extractresult::{ByteSpan, ExtractResult};
