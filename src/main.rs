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
pub mod qualpathreplacer;
pub mod cli_main;
use crate::cli_main::cli_main;
pub mod handle_tools_call;
pub mod remove_spans;
pub mod update_parent_mod;
pub mod dependency_graph;
pub mod split_file;
pub mod merge_spans;
pub mod namevisitor;
pub mod identcollector;
pub mod spans;
pub mod extract;
mod mcp;
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
