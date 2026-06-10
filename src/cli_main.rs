use crate::extract::extract_entity;
use crate::rename_entity::rename_entity;
use crate::split_file::{split_file, split_folder_entities};

fn handle_result<T, E: std::fmt::Display>(r: Result<T, E>, msg: &str) -> T {
    match r {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}: {}", msg, e);
            std::process::exit(1);
        }
    }
}

pub fn cli_main(args: &[String]) {
    if args.is_empty() || args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!("Rust Refactor MCP CLI");
        println!();
        println!("Usage:");
        println!("  Single Entity Extraction:");
        println!("    cargo run -- <file.rs> <EntityName> <target_dir> [--types=struct,fn] [--no-reexport]");
        println!();
        println!("  Batch Tools:");
        println!("    cargo run -- <file.rs> SPLIT <target_dir> [--no-reexport]");
        println!("    cargo run -- SPLIT_DIR <dir_path> [--no-reexport]");
        println!("    cargo run -- . ANALYZE_DEPS <dir_path>");
        println!("    cargo run -- . FIND_DEAD_CODE <dir_path>");
        println!("    cargo run -- . PREFLIGHT <Cargo.toml_path>");
        println!();
        println!("  Transformation Tools:");
        println!("    cargo run -- <file.rs> RENAME <old_name> <new_name>");
        println!("    cargo run -- <file.rs> FORMAT");
        println!("    cargo run -- <file.rs> OPTIMIZE_IMPORTS");
        println!("    cargo run -- <file.rs> SSR <pattern> <replacement>");
        println!("    cargo run -- . EXPAND <target>");
        println!();
        println!("  Options:");
        println!("    --no-reexport    Disable 'pub use' re-exports in the original file.");
        println!("    --types=<types>  Comma-separated list of entity types to include.");
        println!("    --help, -h       Show this help message.");
        return;
    }

    let first = &args[0];

    if first == "SPLIT_DIR" {
        let dir_path = args.get(1).unwrap_or_else(|| {
            eprintln!("Error: Missing dir_path");
            std::process::exit(1);
        });
        let generate_reexport = !args.contains(&"--no-reexport".to_string());
        let entity_types = args.iter().find(|a| a.starts_with("--types=")).map(|a| {
            a.trim_start_matches("--types=")
                .split(',')
                .map(|s| s.to_string())
                .collect()
        });
        handle_result(
            split_folder_entities(dir_path, generate_reexport, entity_types),
            "Split directory failed",
        );
        return;
    }

    let file_path = first;
    let cmd_or_entity = args.get(1).unwrap_or_else(|| {
        eprintln!("Error: Missing entity or command");
        std::process::exit(1);
    });

    if cmd_or_entity == "RENAME" {
        let old_name = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing old_name");
            std::process::exit(1);
        });
        let new_name = args.get(3).unwrap_or_else(|| {
            eprintln!("Error: Missing new_name");
            std::process::exit(1);
        });
        let changed = handle_result(
            rename_entity(file_path, old_name, new_name),
            "Rename failed",
        );
        println!(
            "Renamed {}: {} -> {} (Changed: {})",
            file_path, old_name, new_name, changed
        );
        return;
    }

    if cmd_or_entity == "SPLIT" {
        let target_folder = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing target_folder");
            std::process::exit(1);
        });
        let generate_reexport = !args.contains(&"--no-reexport".to_string());
        let entity_types = args.iter().find(|a| a.starts_with("--types=")).map(|a| {
            a.trim_start_matches("--types=")
                .split(',')
                .map(|s| s.to_string())
                .collect()
        });
        let results = handle_result(
            split_file(
                file_path,
                target_folder,
                None,
                generate_reexport,
                entity_types,
            ),
            "Split failed",
        );
        println!("Split {} into:", file_path);
        for path in results {
            println!("  {}", path);
        }
        return;
    }

    if cmd_or_entity == "FORMAT" {
        let target_file = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing target_file");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::format_code::format_code(target_file),
            "Format failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "OPTIMIZE_IMPORTS" {
        let target_file = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing target_file");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::optimize_imports::optimize_imports(target_file),
            "Optimize imports failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "FIX_CARGO" {
        let manifest_path = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing manifest_path");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::fix_cargo::fix_cargo_errors(manifest_path),
            "Cargo fix failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "EXPAND" {
        let target = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing expand target");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::macro_expander::expand_macros(target),
            "Macro expansion failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "ANALYZE_DEPS" {
        let dir_path = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing dir_path");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::dependency_graph_analyzer::analyze_dependencies(dir_path),
            "Analysis failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "FIND_DEAD_CODE" {
        let dir_path = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing dir_path");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::dead_code_finder::find_dead_code(dir_path),
            "Dead code analysis failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "PREFLIGHT" {
        let manifest_path = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing manifest_path");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::preflight_validator::validate_project(manifest_path),
            "Preflight validation failed",
        );
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "SSR" {
        let pattern = args.get(2).unwrap_or_else(|| {
            eprintln!("Error: Missing pattern");
            std::process::exit(1);
        });
        let replacement = args.get(3).unwrap_or_else(|| {
            eprintln!("Error: Missing replacement");
            std::process::exit(1);
        });
        let result = handle_result(
            crate::ssr::ssr(file_path, pattern, replacement),
            "SSR failed",
        );
        println!(
            "SSR on {}: {} -> {} (Changed: {})",
            file_path, pattern, replacement, result
        );
        return;
    }

    let entity_name = cmd_or_entity;
    let target_folder = args.get(2).unwrap_or_else(|| {
        eprintln!("Error: Missing target_folder");
        std::process::exit(1);
    });
    let entity_types = args.iter().find(|a| a.starts_with("--types=")).map(|a| {
        a.trim_start_matches("--types=")
            .split(',')
            .map(|s| s.to_string())
            .collect()
    });
    let generate_reexport = !args.contains(&"--no-reexport".to_string());

    let source = std::fs::read_to_string(file_path).unwrap_or_else(|e| {
        eprintln!("Error: Cannot read file {}: {}", file_path, e);
        std::process::exit(1);
    });
    let result = handle_result(
        extract_entity(
            &source,
            entity_name,
            target_folder,
            entity_types,
            Some(file_path),
            None,
            generate_reexport,
        ),
        "Extraction failed",
    );
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
