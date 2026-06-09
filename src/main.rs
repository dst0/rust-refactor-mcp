pub mod cli_main;
pub mod collect_use_names;
pub mod extract_module_path;
pub mod format_ty_name;
pub mod handle_initialize;
pub mod handle_tools_list;
pub mod has_use_ref;
pub mod jsonrpcrequest;
pub mod line_col_to_byte;
pub mod qualpathreplacer;
pub mod run_server;
pub mod span_to_byte;
use crate::cli_main::cli_main;
pub mod dead_code_finder;
pub mod dependency_graph;
pub mod dependency_graph_analyzer;
pub mod extract;
pub mod extractresult;
pub mod fix_cargo;
pub mod format_code;
pub mod handle_tools_call;
pub mod identcollector;
pub mod macro_expander;
mod mcp;
pub mod merge_spans;
pub mod namevisitor;
pub mod optimize_imports;
pub mod preflight_validator;
pub mod rename_entity;
pub mod spans;
pub mod split_file;
pub mod ssr;
pub mod update_parent_mod;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && (args[1] == "SPLIT_DIR" || args.len() >= 4) {
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
