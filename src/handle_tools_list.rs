use serde_json::{json, Value};
pub fn handle_tools_list(id: &Option<Value>) -> Value {
    json!(
        { "jsonrpc" : "2.0", "id" : id, "result" : { "tools" : [
        { "name" : "extract_entity", "description" : "Extract a named entity from a Rust source file.", "inputSchema" : { "type" : "object", "properties" : { "file_path" : { "type" : "string" }, "entity_name" : { "type" : "string" }, "target_folder" : { "type" : "string" }, "entity_types" : { "type" : "array", "items": { "type": "string" } }, "generate_reexport": { "type": "boolean" } }, "required" : ["file_path", "entity_name", "target_folder"] } },
        { "name" : "format_code", "description" : "Format a Rust file.", "inputSchema" : { "type" : "object", "properties" : { "file_path" : { "type" : "string" } }, "required" : ["file_path"] } },
        { "name" : "rename_entity", "description" : "Rename an entity.", "inputSchema" : { "type" : "object", "properties" : { "file_path" : { "type" : "string" }, "old_name" : { "type" : "string" }, "new_name" : { "type" : "string" } }, "required" : ["file_path", "old_name", "new_name"] } },
        { "name" : "fix_cargo_errors", "description" : "Run cargo fix.", "inputSchema" : { "type" : "object", "properties" : { "manifest_path" : { "type" : "string" } }, "required" : ["manifest_path"] } },
        { "name" : "optimize_imports", "description" : "Optimize imports.", "inputSchema" : { "type" : "object", "properties" : { "file_path" : { "type" : "string" } }, "required" : ["file_path"] } },
        { "name" : "ssr", "description" : "Structural search and replace.", "inputSchema" : { "type" : "object", "properties" : { "file_path" : { "type" : "string" }, "pattern" : { "type" : "string" }, "replacement" : { "type" : "string" } }, "required" : ["file_path", "pattern", "replacement"] } },
        { "name" : "expand_macros", "description" : "Expand macros.", "inputSchema" : { "type" : "object", "properties" : { "target" : { "type" : "string" } }, "required" : ["target"] } },
        { "name" : "analyze_dependencies", "description" : "Analyze module dependencies and coupling.", "inputSchema" : { "type" : "object", "properties" : { "dir_path" : { "type" : "string" } }, "required" : ["dir_path"] } },
        { "name" : "find_dead_code", "description" : "Find potentially dead code in a directory.", "inputSchema" : { "type" : "object", "properties" : { "dir_path" : { "type" : "string" } }, "required" : ["dir_path"] } },
        { "name" : "preflight_validator", "description" : "Run preflight checks (cargo check/test).", "inputSchema" : { "type" : "object", "properties" : { "manifest_path" : { "type" : "string" } }, "required" : ["manifest_path"] } },
        { "name" : "split_folder_entities", "description" : "Split folder entities.", "inputSchema" : { "type" : "object", "properties" : { "dir_path" : { "type" : "string" }, "generate_reexport": { "type": "boolean" }, "entity_types": { "type": "array", "items": { "type": "string" } } }, "required" : ["dir_path"] } },
        { "name" : "discover_multi_entity_files", "description" : "Discover multi-entity files.", "inputSchema" : { "type" : "object", "properties" : { "dir_path" : { "type" : "string" } }, "required" : ["dir_path"] } }
        ] } }
    )
}
