use serde_json::{json, Value};
pub fn handle_initialize(id: &Option<Value>) -> Value {
    json!(
        { "jsonrpc" : "2.0", "id" : id, "result" : { "protocolVersion" : "2024-11-05",
        "capabilities" : { "tools" : {} }, "serverInfo" : { "name" : "rust-refactor-mcp",
        "version" : "0.1.0" } } }
    )
}
