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
        .block_on(mcp::run_server())
    {
        eprintln!("Server error: {}", e);
    }
}
