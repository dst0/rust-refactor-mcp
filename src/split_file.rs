use crate::dependency_graph::DependencyGraph;
use crate::extract::{extract_entity, get_item_name, item_type, to_snake_case};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use syn::{parse_file, Item};

pub fn split_file(
    file_path: &str,
    target_folder: &str,
    cached_files: Option<&Vec<PathBuf>>,
    generate_reexport: bool,
    entity_types: Option<Vec<String>>,
    fix_vis: Option<&str>,
    fix_macros: Option<&str>,
) -> Result<Vec<String>, String> {
    let initial_source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let parsed = parse_file(&initial_source).map_err(|e| e.to_string())?;
    let graph = DependencyGraph::build(&parsed);

    // Basic heuristic: entities with fewer dependencies first
    let item_names: Vec<String> = parsed
        .items
        .iter()
        .filter_map(|item| {
            if matches!(item, Item::Use(_) | Item::ExternCrate(_) | Item::Macro(_)) {
                return None;
            }
            get_item_name(item)
        })
        .collect();

    // Deduplicate while preserving order of first appearance
    let mut unique_names = Vec::new();
    let mut seen = HashSet::new();
    for name in item_names {
        if seen.insert(name.clone()) {
            unique_names.push(name);
        }
    }

    unique_names.sort_by(|a, b| {
        let a_deps = graph.deps.get(a).map(|s| s.len()).unwrap_or(0);
        let b_deps = graph.deps.get(b).map(|s| s.len()).unwrap_or(0);
        a_deps.cmp(&b_deps)
    });

    let source_stem = PathBuf::from(file_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut extracted_files = Vec::new();
    let total_unique = unique_names.len();
    for (idx, name) in unique_names.into_iter().enumerate() {
        // Skip if the entity name matches the current file name (already "split")
        if to_snake_case(&name) == source_stem {
            continue;
        }

        println!("    [{}/{}] Extracting {}...", idx + 1, total_unique, name);
        // Re-read source from disk because previous extraction modified it
        let current_source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;

        // We need to know the type for the first item matching this name
        let current_parsed = parse_file(&current_source).map_err(|e| e.to_string())?;
        let items: Vec<Item> = current_parsed
            .items
            .iter()
            .filter(|item| get_item_name(item).as_deref() == Some(&name))
            .cloned()
            .collect();

        if items.is_empty() {
            // Might have been extracted as part of another entity (e.g. if it was an impl)
            continue;
        }

        let itypes: Vec<String> = items
            .iter()
            .map(|item| item_type(item).to_string())
            .collect();

        // Filter by types if provided
        if let Some(ref filter_types) = entity_types {
            if !itypes.iter().any(|t| filter_types.contains(t)) {
                continue;
            }
        }

        let result = extract_entity(
            &current_source,
            &name,
            target_folder,
            Some(itypes),
            Some(file_path),
            cached_files,
            generate_reexport,
            fix_vis,
            fix_macros,
        );

        match result {
            Ok(res) => {
                extracted_files.push(res.new_file_path);
            }
            Err(e) => {
                println!("Warning: Failed to extract {}: {}", name, e);
            }
        }
    }
    Ok(extracted_files)
}

pub fn discover_multi_entity_files(dir_path: &str) -> Result<HashMap<PathBuf, usize>, String> {
    let mut files = Vec::new();
    collect_rs_files_internal(PathBuf::from(dir_path), &mut files);
    let mut multi_entity = HashMap::new();

    for file in files {
        let file_str = fs::read_to_string(&file).map_err(|e| e.to_string())?;
        if let Ok(parsed) = parse_file(&file_str) {
            let count = parsed
                .items
                .iter()
                .filter(|item| {
                    !matches!(item, Item::Use(_) | Item::ExternCrate(_) | Item::Macro(_))
                })
                .count();
            if count > 1 {
                multi_entity.insert(file, count);
            }
        }
    }
    Ok(multi_entity)
}

pub fn split_folder_entities(
    dir_path: &str,
    generate_reexport: bool,
    entity_types: Option<Vec<String>>,
    fix_vis: Option<&str>,
    fix_macros: Option<&str>,
) -> Result<(), String> {
    println!("Discovering multi-entity files in {}...", dir_path);
    let multi_entity = discover_multi_entity_files(dir_path)?;
    println!("Found {} files with multiple entities.", multi_entity.len());

    let all_files: Vec<PathBuf> = multi_entity.keys().cloned().collect();
    let total_files = multi_entity.len();

    for (idx, (file, count)) in multi_entity.into_iter().enumerate() {
        let file_str = file.to_string_lossy().to_string();
        let target_folder = file.parent().unwrap().to_string_lossy().to_string();

        // Skip entry points
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        if matches!(stem, "lib" | "mod" | "main") {
            continue;
        }

        print!(
            "\r[{}/{}] Splitting {} ({} entities)...          ",
            idx + 1,
            total_files,
            file_str,
            count
        );
        use std::io::Write;
        std::io::stdout().flush().ok();

        match split_file(
            &file_str,
            &target_folder,
            Some(&all_files),
            generate_reexport,
            entity_types.clone(),
            fix_vis,
            fix_macros,
        ) {
            Ok(extracted) => {
                if !extracted.is_empty() {
                    println!(
                        "\nProcessed {}: extracted {} entities",
                        file_str,
                        extracted.len()
                    );
                }
            }
            Err(e) => {
                println!("\nError splitting {}: {}", file_str, e);
            }
        }
    }
    Ok(())
}

pub fn collect_rs_files_internal(dir: PathBuf, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().and_then(|s| s.to_str()) == Some("target") {
                    continue;
                }
                collect_rs_files_internal(path, files);
            } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
                files.push(path);
            }
        }
    }
}
