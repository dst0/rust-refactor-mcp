pub mod namevisitor;
pub mod identcollector;
pub mod bytespan;
pub mod extractresult;
pub mod spans;
mod extract;
mod mcp;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 4 {
        cli_main(&args[1..]);
        return;
    }
    if let Err(e) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(mcp::run_server())
    {
        eprintln!("Server error: {}", e);
    }
}
fn cli_main(args: &[String]) {
    let file_path = &args[0];
    let entity_name = &args[1];
    let target_folder = &args[2];
    let entity_type = if args.len() > 3 { Some(args[3].as_str()) } else { None };
    let source = std::fs::read_to_string(file_path).expect("Cannot read file");
    let result = extract::extract_entity(
            &source,
            entity_name,
            target_folder,
            entity_type,
            Some(file_path),
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
