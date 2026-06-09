use crate::extract::extract_entity;
use crate::split_file::{split_file, split_directory};
pub fn cli_main(args: &[String]) {
    let file_path = &args[0];

    if file_path == "SPLIT_DIR" {
        let dir_path = &args[1];
        split_directory(dir_path).expect("Split directory failed");
        return;
    }

    let cmd_or_entity = &args[1];
    let target_folder = &args[2];

    if cmd_or_entity == "FORMAT" {
        let file_path = &args[2];
        let result = crate::format_code::format_code(file_path).expect("Format failed");
        println!("{}", result);
        return;
    }

    if cmd_or_entity == "SPLIT" {
        let results = split_file(file_path, target_folder, None).expect("Split failed");
        println!("Split {} into:", file_path);
        for path in results {
            println!("  {}", path);
        }
        return;
    }

    let entity_name = cmd_or_entity;
    let entity_type = if args.len() > 3 {
        Some(args[3].as_str())
    } else {
        None
    };
    let source = std::fs::read_to_string(file_path).expect("Cannot read file");
    let result = extract_entity(
            &source,
            entity_name,
            target_folder,
            entity_type,
            Some(file_path),
            None,
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
