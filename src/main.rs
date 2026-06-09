pub mod collapse_blank_lines;
pub mod jsonrpcrequest;
pub mod handle_tools_list;
pub mod handle_initialize;
pub mod run_server;
pub mod collect_use_names;
pub mod extract_module_path;
pub mod has_use_ref;
pub mod line_col_to_byte;
pub mod span_to_byte;
pub mod format_ty_name;
pub mod item_type;
pub mod get_item_name;
pub mod make_item_pub;
pub mod qualpathreplacer;
pub mod find_extracted_indices;
pub mod collect_referenced_identifiers;
pub mod is_import_used;
pub mod cleanup_imports_in_ast;
pub mod detect_needed_imports_for_extracted;
pub mod detect_cross_refs_for_extracted;
pub mod update_usage_files;
pub mod cli_main;
use crate::cli_main::cli_main;
pub mod handle_tools_call;
pub mod remove_spans;
pub mod cleanup_unused_imports;
pub mod update_parent_mod;
pub mod merge_spans;
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
        .block_on(run_server::run_server())
    {
        eprintln!("Server error: {}", e);
    }
}
