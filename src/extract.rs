use crate::identcollector::IdentCollector;
use crate::merge_spans::merge_spans;
use crate::update_parent_mod::update_parent_mod;
use proc_macro2::Span;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{File, Item, ItemFn, ItemUse, Type, UseTree};
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
            
            let mod_ident = syn::Ident::new(&to_snake_case(entity_name), proc_macro2::Span::call_site());
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
    let new_module = to_snake_case(entity_name);
    let source_stem = source_file_path.as_ref().and_then(|p| {
        PathBuf::from(p)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
    });
    let same_file = source_stem.as_deref() == Some(&new_module);
    let new_path = PathBuf::from(target_folder).join(format!("{}.rs", new_module));
    let new_file_path = if same_file {
        source_file_path.map(|p| p.to_string()).unwrap_or_default()
    } else {
        new_path.to_string_lossy().to_string()
    };
    if !same_file {
        let needed_imports = detect_needed_imports_for_extracted(&parsed, &extracted, entity_name);
        let mut new_file = File {
            shebang: parsed.shebang.clone(),
            attrs: parsed.attrs.clone(),
            items: Vec::new(),
        };
        for imp in &needed_imports {
            new_file.items.push(Item::Use(imp.clone()));
        }
        for item in &extracted {
            let mut item = item.clone();
            make_item_pub(&mut item);
            new_file.items.push(item);
        }
        for promo in macro_promotions {
            new_file.items.push(promo);
        }
        let cross_refs =
            detect_cross_refs_for_extracted(&parsed, &extracted, entity_name, source_file_path);
        for imp in cross_refs {
            new_file.items.push(Item::Use(imp));
        }
        let filename = format!("{}.rs", new_module);
        let new_path = PathBuf::from(target_folder).join(&filename);
        fs::create_dir_all(PathBuf::from(target_folder))
            .map_err(|e| format!("Cannot create dir: {}", e))?;
        let content = prettyplease::unparse(&new_file);
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
            let full_mod_path = get_full_module_path(target_folder, &new_module);
            let vis_prefix = if is_pub { "pub " } else { "" };
            let escaped_entity = escape_path_segment(entity_name);
            let use_str = format!("{}use {}::{};", vis_prefix, full_mod_path, escaped_entity);
            if let Ok(source_use) = syn::parse_str::<Item>(&use_str) {
                remaining.insert(0, source_use);
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
            source_stem.as_deref(),
            source_file_path,
            cached_files,
        )?;
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
    _entity_name: &str,
) -> Vec<ItemUse> {
    parsed
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Use(iu) = item {
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
                    p.ident = syn::Ident::new("crate", p.ident.span());
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
            PathBuf::from(p)
                .file_stem()
                .and_then(|s| s.to_str().map(escape_path_segment))
        })
        .unwrap_or_else(|| "super".to_string());
    let escaped_names: Vec<String> = needed.iter().map(|n| escape_path_segment(n)).collect();
    let names = escaped_names.join(", ");
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

        let mut parsed = match syn::parse_file(&file_content) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let new_module = to_snake_case(entity_name);
        let mut changed = false;

        // Handle qualified calls like mcp::run_server
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
                            let escaped_name = escape_path_segment(name);
                            let use_str = format!("use {}::{};", prefix, escaped_name);
                            if let Ok(parsed_use) = syn::parse_str::<ItemUse>(&use_str) {
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
            let full_mod_path = get_full_module_path(target_folder, &new_module);
            let escaped_entity = escape_path_segment(entity_name);
            let use_str = format!("use {}::{};", full_mod_path, escaped_entity);
            let new_use: Item = syn::parse_str(&use_str).unwrap();
            new_items.insert(0, new_use);
            let used = collect_referenced_identifiers(&new_items);
            new_items.retain(|item| {
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
    if !matches!(get_item_vis(item), Some(syn::Visibility::Inherited))
        && get_item_vis(item).is_some()
    {
        return;
    }
    let pub_vis = syn::Visibility::Public(syn::token::Pub::default());
    match item {
        Item::Fn(f) => f.vis = pub_vis,
        Item::Struct(s) => {
            s.vis = pub_vis.clone();
            match &mut s.fields {
                syn::Fields::Named(fields) => {
                    for field in &mut fields.named {
                        if matches!(field.vis, syn::Visibility::Inherited) {
                            field.vis = pub_vis.clone();
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    for field in &mut fields.unnamed {
                        if matches!(field.vis, syn::Visibility::Inherited) {
                            field.vis = pub_vis.clone();
                        }
                    }
                }
                syn::Fields::Unit => {}
            }
        }
        Item::Enum(e) => {
            e.vis = pub_vis.clone();
            for variant in &mut e.variants {
                match &mut variant.fields {
                    syn::Fields::Named(fields) => {
                        for field in &mut fields.named {
                            if matches!(field.vis, syn::Visibility::Inherited) {
                                field.vis = pub_vis.clone();
                            }
                        }
                    }
                    syn::Fields::Unnamed(fields) => {
                        for field in &mut fields.unnamed {
                            if matches!(field.vis, syn::Visibility::Inherited) {
                                field.vis = pub_vis.clone();
                            }
                        }
                    }
                    syn::Fields::Unit => {}
                }
            }
        }
        Item::Trait(t) => t.vis = pub_vis,
        Item::Type(t) => t.vis = pub_vis,
        Item::Const(c) => c.vis = pub_vis,
        Item::Static(s) => s.vis = pub_vis,
        Item::Mod(m) => m.vis = pub_vis,
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
        Item::Fn(f) => Some(f.sig.ident.to_string()),
        Item::Struct(s) => Some(s.ident.to_string()),
        Item::Enum(e) => Some(e.ident.to_string()),
        Item::Trait(t) => Some(t.ident.to_string()),
        Item::Type(t) => Some(t.ident.to_string()),
        Item::Const(c) => Some(c.ident.to_string()),
        Item::Static(s) => Some(s.ident.to_string()),
        _ => None,
    }
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
        if i.segments.len() == 2
            && i.segments[0].ident == self.old_mod
            && i.segments[1].ident == self.entity_name
        {
            i.segments[0].ident = syn::Ident::new(&self.new_mod, i.segments[0].ident.span());
            self.changed = true;
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

pub fn escape_path_segment(s: &str) -> String {
    if is_keyword(s) && s != "crate" && s != "self" && s != "super" && s != "Self" {
        format!("r#{}", s)
    } else {
        s.to_string()
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
