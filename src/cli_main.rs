use crate::extract::extract_entity;
use crate::split_file::{split_file, split_directory};

pub fn cli_main(args: &[String]) {
    if args.is_empty() {
        panic!("Missing command/file_path");
    }

    let first = &args[0];

    if first == "SPLIT_DIR" {
        let dir_path = args.get(1).expect("Missing dir_path");
        let generate_reexport = !args.contains(&"--no-reexport".to_string());
        split_directory(dir_path, generate_reexport).expect("Split directory failed");
        return;
    }

    let file_path = first;
    let cmd_or_entity = args.get(1).expect("Missing entity or command");
    
    if cmd_or_entity == "SPLIT" {
        let target_folder = args.get(2).expect("Missing target_folder");
        let generate_reexport = !args.contains(&"--no-reexport".to_string());
        let results = split_file(file_path, target_folder, None, generate_reexport).expect("Split failed");
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

    let entity_name = cmd_or_entity;
    let target_folder = args.get(2).expect("Missing target_folder");
    let entity_type = args.get(3).map(|s| s.as_str());
    let generate_reexport = !args.contains(&"--no-reexport".to_string());

    let source = std::fs::read_to_string(file_path).expect("Cannot read file");
    let result = extract_entity(
            &source,
            entity_name,
            target_folder,
            entity_type,
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
        println!("Usage files updated: {}", result.usage_files_updated.join(", "));
    }
}
