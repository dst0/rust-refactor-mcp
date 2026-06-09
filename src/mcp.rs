use serde::Deserialize;
use crate::handle_tools_call::handle_tools_call;
use std::io::{BufRead, BufReader, Write};
use serde_json::{json, Value};
/// Run the MCP stdio server.
pub async fn run_server() -> Result<(), String> {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let reader = BufReader::new(stdin);
    for line in reader.lines() {
        let line = line.map_err(|e| format!("Read error: {}", e))?;
        let request: JsonRpcRequest = serde_json::from_str(&line)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        if request.method == "initialize" {
            eprintln!("rust-refactor-mcp server started");
        }
        let response = match request.method.as_str() {
            "initialize" => handle_initialize(&request.id),
            "initialized" => continue,
            "tools/list" => handle_tools_list(&request.id),
            "tools/call" => {
                handle_tools_call(
                    &request.id,
                    request.params.as_ref().ok_or("Missing params for tools/call")?,
                )?
            }
            _ => {
                json!(
                    { "jsonrpc" : "2.0", "id" : request.id, "error" : { "code" : - 32601,
                    "message" : format!("Method not found: {}", request.method) } }
                )
            }
        };
        let mut stdout = stdout.lock();
        writeln!(
            stdout, "{}", serde_json::to_string(& response).map_err(| e | e.to_string())
            ?
        )
            .map_err(|e| format!("Write error: {}", e))?;
        stdout.flush().map_err(|e| format!("Flush error: {}", e))?;
    }
    Ok(())
}
pub fn handle_initialize(id: &Option<Value>) -> Value {
    json!(
        { "jsonrpc" : "2.0", "id" : id, "result" : { "protocolVersion" : "2024-11-05",
        "capabilities" : { "tools" : {} }, "serverInfo" : { "name" : "rust-refactor-mcp",
        "version" : "0.1.0" } } }
    )
}
pub fn handle_tools_list(id: &Option<Value>) -> Value {
    json!(
        { "jsonrpc" : "2.0", "id" : id, "result" : { "tools" : [{ "name" :
        "extract_entity", "description" :
        "Extract a named entity (struct, enum, trait, fn, impl) from a Rust source file into its own module file. Related impl blocks and test modules are extracted together. Cross-file use statements are updated automatically.",
        "inputSchema" : { "type" : "object", "properties" : { "file_path" : { "type" :
        "string", "description" : "Path to the Rust source file containing the entity" },
        "entity_name" : { "type" : "string", "description" :
        "Name of the entity to extract" }, "target_folder" : { "type" : "string",
        "description" : "Directory where the new module file will be written" },
        "entity_type" : { "type" : "string", "description" :
        "Hint for entity type: struct, enum, fn, trait, impl", "enum" : ["struct",
        "enum", "fn", "trait", "impl"] } }, "required" : ["file_path", "entity_name",
        "target_folder"] } }] } }
    )
}
#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    #[serde(default)]
    id: Option<Value>,
    #[serde(default)]
    params: Option<Value>,
}
