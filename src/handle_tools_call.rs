use serde_json::{json, Value};
pub fn handle_tools_call(id: &Option<Value>, params: &Value) -> Result<Value, String> {
    let name = params.get("name").and_then(Value::as_str).ok_or("Missing tool name")?;
    let args = params.get("arguments").ok_or("Missing arguments")?;
    
    match name {
        "extract_entity" => {
            let file_path = args.get("file_path").and_then(Value::as_str).ok_or("file_path is required")?;
            let entity_name = args.get("entity_name").and_then(Value::as_str).ok_or("entity_name is required")?;
            let target_folder = args.get("target_folder").and_then(Value::as_str).ok_or("target_folder is required")?;
            let entity_type = args.get("entity_type").and_then(Value::as_str);
            let generate_reexport = args.get("generate_reexport").and_then(Value::as_bool).unwrap_or(true);
            let source = std::fs::read_to_string(file_path).map_err(|e| format!("Cannot read file: {}", e))?;
            let result = crate::extract::extract_entity(
                &source,
                entity_name,
                target_folder,
                entity_type,
                Some(file_path),
                None,
                generate_reexport,
            )?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : format!("Extracted {} → {}\nItems: {}", entity_name, result.new_file_path, result.items_extracted.join(", ")) }] } }))
        }
        "format_code" => {
            let file_path = args.get("file_path").and_then(Value::as_str).ok_or("file_path is required")?;
            let result = crate::format_code::format_code(file_path)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : result }] } }))
        }
        _ => Err(format!("Unknown tool: {}", name)),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::handle_initialize::handle_initialize;
    use crate::handle_tools_list::handle_tools_list;
    #[test]
    fn initialize_response() {
        let resp = handle_initialize(&None);
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["result"] ["serverInfo"] ["name"], "rust-refactor-mcp");
        assert!(resp["result"] ["capabilities"] ["tools"].is_object());
    }
    #[test]
    fn tools_list() {
        let resp = handle_tools_list(&None);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0] ["name"], "extract_entity");
    }
    #[test]
    fn tools_call_missing_file_path() {
        let params = json!(
            { "name" : "extract_entity", "arguments" : { "entity_name" : "Foo",
            "target_folder" : "." } }
        );
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("file_path"));
    }
    #[test]
    fn tools_call_missing_entity_name() {
        let params = json!(
            { "name" : "extract_entity", "arguments" : { "file_path" : "test.rs",
            "target_folder" : "." } }
        );
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("entity_name"));
    }
    #[test]
    fn tools_call_missing_target_folder() {
        let params = json!(
            { "name" : "extract_entity", "arguments" : { "file_path" : "test.rs",
            "entity_name" : "Foo" } }
        );
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("target_folder"));
    }
    #[test]
    fn tools_call_missing_arguments() {
        let params = json!({ "name" : "extract_entity" });
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("Missing arguments"));
    }
    #[test]
    fn tools_call_invalid_file() {
        let params = json!(
            { "name" : "extract_entity", "arguments" : { "file_path" : "nonexistent.rs",
            "entity_name" : "Foo", "target_folder" : "." } }
        );
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("Cannot read file"));
    }
    #[test]
    fn tools_call_success() {
        let tmp = std::env::temp_dir().join("mcp_test");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let file_path = tmp.join("source.rs");
        std::fs::write(&file_path, "struct Foo { x: i32 }\nfn bar() {}").unwrap();
        let params = json!(
            { "name" : "extract_entity", "arguments" : { "file_path" : file_path
            .to_string_lossy(), "entity_name" : "Foo", "target_folder" : tmp
            .to_string_lossy() } }
        );
        let result = handle_tools_call(&None, &params).unwrap();
        assert!(
            result["result"] ["content"] [0] ["text"].as_str().unwrap().contains("Foo")
        );
        assert!(result["result"] ["structuredContent"] ["new_file_path"].is_string());
        std::fs::remove_dir_all(&tmp).ok();
    }
}
