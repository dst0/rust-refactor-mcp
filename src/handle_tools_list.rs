use serde_json::{json, Value};
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
