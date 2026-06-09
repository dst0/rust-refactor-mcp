use crate::extract::extract_entity;
use crate::rename_entity::rename_entity;
use crate::split_file::{split_file, split_folder_entities};

pub fn cli_main(args: &[String]) {
    if args.is_empty() {
        panic!("Missing command/file_path");
    }

    let first = &args[0];

    if first == "SPLIT_DIR" {
        let dir_path = args.get(1).expect("Missing dir_path");
        let generate_reexport = !args.contains(&"--no-reexport".to_string());
        let entity_types = args.iter().find(|a| a.starts_with("--types=")).map(|a| {
            a.trim_start_matches("--types=")
                .split(',')
                .map(|s| s.to_string())
                .collect()
        });
        split_folder_entities(dir_path, generate_reexport, entity_types)
            .expect("Split directory failed");
        return;
    }

    let file_path = first;
    let cmd_or_entity = args.get(1).expect("Missing entity or command");

    if cmd_or_entity == "RENAME" {
        let old_name = args.get(2).expect("Missing old_name");
        let new_name = args.get(3).expect("Missing new_name");
        let changed = rename_entity(file_path, old_name, new_name).expect("Rename failed");
        println!(
            "Renamed {}: {} -> {} (Changed: {})",
            file_path, old_name, new_name, changed
        );
        return;
    }

    if cmd_or_entity == "SPLIT" {
        let target_folder = args.get(2).expect("Missing target_folder");
        let generate_reexport = !args.contains(&"--no-reexport".to_string());
        let entity_types = args.iter().find(|a| a.starts_with("--types=")).map(|a| {
            a.trim_start_matches("--types=")
                .split(',')
                .map(|s| s.to_string())
                .collect()
        });
        let results = split_file(
            file_path,
            target_folder,
            None,
            generate_reexport,
            entity_types,
        )
        .expect("Split failed");
        println!("Split {} into:", file_path);
        for path in results {
            println!("  {}", path);
        }
        return;
    }

    if cmd_or_entity == "FORMAT" {
        let target_file = args.get(2).expect("Missing target_file");
        let result = crate::format_code::format_code(target_file).expect("Format failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "OPTIMIZE_IMPORTS" {
        let target_file = args.get(2).expect("Missing target_file");
        let result = crate::optimize_imports::optimize_imports(target_file)
            .expect("Optimize imports failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "FIX_CARGO" {
        let manifest_path = args.get(2).expect("Missing manifest_path");
        let result = crate::fix_cargo::fix_cargo_errors(manifest_path).expect("Cargo fix failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "EXPAND" {
        let target = args.get(2).expect("Missing expand target");
        let result = crate::macro_expander::expand_macros(target).expect("Macro expansion failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "ANALYZE_DEPS" {
        let dir_path = args.get(2).expect("Missing dir_path");
        let result = crate::dependency_graph_analyzer::analyze_dependencies(dir_path)
            .expect("Analysis failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "FIND_DEAD_CODE" {
        let dir_path = args.get(2).expect("Missing dir_path");
        let result =
            crate::dead_code_finder::find_dead_code(dir_path).expect("Dead code analysis failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "PREFLIGHT" {
        let manifest_path = args.get(2).expect("Missing manifest_path");
        let result = crate::preflight_validator::validate_project(manifest_path)
            .expect("Preflight validation failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "SSR" {
        let pattern = args.get(2).expect("Missing pattern");
        let replacement = args.get(3).expect("Missing replacement");
        let result = crate::ssr::ssr(file_path, pattern, replacement).expect("SSR failed");
        println!(
            "SSR on {}: {} -> {} (Changed: {})",
            file_path, pattern, replacement, result
        );
        return;
    }

    let entity_name = cmd_or_entity;
    let target_folder = args.get(2).expect("Missing target_folder");
    let entity_types = args.iter().find(|a| a.starts_with("--types=")).map(|a| {
        a.trim_start_matches("--types=")
            .split(',')
            .map(|s| s.to_string())
            .collect()
    });
    let generate_reexport = !args.contains(&"--no-reexport".to_string());

    let source = std::fs::read_to_string(file_path).expect("Cannot read file");
    let result = extract_entity(
        &source,
        entity_name,
        target_folder,
        entity_types,
        Some(file_path),
        None,
        generate_reexport,
    )
    .expect("Extraction failed");
    println!("Extracted {} → {}", entity_name, result.new_file_path);
    println!("Items: {}", result.items_extracted.join(", "));
    if let Some(test_path) = &result.test_file_path {
        println!("Tests → {}", test_path);
    }
    if !result.usage_files_updated.is_empty() {
        println!(
            "Usage files updated: {}",
            result.usage_files_updated.join(", ")
        );
    }
}
