use crate::handle_initialize::handle_initialize;
use crate::handle_tools_call::handle_tools_call;
use crate::handle_tools_list::handle_tools_list;
use crate::jsonrpcrequest::JsonRpcRequest;
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
/// Run the MCP stdio server.
pub async fn run_server() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let reader = BufReader::new(stdin);
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        let request: JsonRpcRequest =
            serde_json::from_str(&line).map_err(|e| format!("Invalid JSON: {}", e))?;
        if request.method == "initialize" {
            eprintln!("rust-refactor-mcp server started");
        }
        let response = match request.method.as_str() {
            "initialize" => handle_initialize(&request.id),
            "initialized" => continue,
            "tools/list" => handle_tools_list(&request.id),
            "tools/call" => handle_tools_call(
                &request.id,
                request
                    .params
                    .as_ref()
                    .ok_or("Missing params for tools/call")?,
            )?,
            _ => {
                json!(
                    { "jsonrpc" : "2.0", "id" : request.id, "error" : { "code" : - 32601,
                    "message" : format!("Method not found: {}", request.method) } }
                )
            }
        };
        let mut stdout = stdout.lock();
        writeln!(
            stdout,
            "{}",
            serde_json::to_string(&response).map_err(|e| e.to_string())?
        )
        .map_err(|e| format!("Write error: {}", e))?;
        stdout.flush().map_err(|e| format!("Flush error: {}", e))?;
    }
    Ok(())
}
