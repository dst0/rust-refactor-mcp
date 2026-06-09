use crate::extract::{extract_entity, get_item_name, item_type, to_snake_case};
use crate::dependency_graph::DependencyGraph;
use syn::{parse_file, Item};
use std::fs;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn split_file(
    file_path: &str, 
    target_folder: &str,
    cached_files: Option<&Vec<PathBuf>>,
    generate_reexport: bool,
) -> Result<Vec<String>, String> {
    let initial_source = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let parsed = parse_file(&initial_source).map_err(|e| e.to_string())?;
    let graph = DependencyGraph::build(&parsed);
    
    // Basic heuristic: entities with fewer dependencies first
    let item_names: Vec<String> = parsed.items.iter()
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
        let itype = current_parsed.items.iter()
            .find(|item| get_item_name(item).as_deref() == Some(&name))
            .map(|item| item_type(item));

        if itype.is_none() {
            // Might have been extracted as part of another entity (e.g. if it was an impl)
            continue;
        }

        let result = extract_entity(
            &current_source, 
            &name, 
            target_folder, 
            itype, 
            Some(file_path),
            cached_files,
            generate_reexport,
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

pub fn split_directory(dir_path: &str, generate_reexport: bool) -> Result<(), String> {
    println!("Scanning directory {}...", dir_path);
    let mut files = Vec::new();
    collect_rs_files_internal(PathBuf::from(dir_path), &mut files);
    println!("Found {} .rs files.", files.len());
    
    let total_files = files.len();
    for (idx, file) in files.iter().enumerate() {
        let file_str = file.to_string_lossy().to_string();
        let target_folder = file.parent().unwrap().to_string_lossy().to_string();
        
        // Skip entry points
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
        if matches!(stem, "lib" | "mod" | "main") {
            continue;
        }

        print!("\r[{}/{}] Processing {}...          ", idx + 1, total_files, file_str);
        use std::io::Write;
        std::io::stdout().flush().ok();
        match split_file(&file_str, &target_folder, Some(&files), generate_reexport) {
            Ok(extracted) => {
                if !extracted.is_empty() {
                    println!("Processed {}: extracted {} entities", file_str, extracted.len());
                }
            }
            Err(e) => {
                println!("Error splitting {}: {}", file_str, e);
            }
        }
    }
    Ok(())
}

fn collect_rs_files_internal(dir: PathBuf, files: &mut Vec<PathBuf>) {
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
