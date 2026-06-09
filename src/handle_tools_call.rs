use serde_json::{json, Value};
use std::collections::HashMap;

pub fn handle_tools_call(id: &Option<Value>, params: &Value) -> Result<Value, String> {
    let name = params.get("name").and_then(Value::as_str).ok_or("Missing tool name")?;
    let args = params.get("arguments").ok_or("Missing arguments")?;
    
    match name {
        "extract_entity" => {
            let file_path = args.get("file_path").and_then(Value::as_str).ok_or("file_path is required")?;
            let entity_name = args.get("entity_name").and_then(Value::as_str).ok_or("entity_name is required")?;
            let target_folder = args.get("target_folder").and_then(Value::as_str).ok_or("target_folder is required")?;
            let entity_types = args.get("entity_types").and_then(Value::as_array).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
            let generate_reexport = args.get("generate_reexport").and_then(Value::as_bool).unwrap_or(true);
            let source = std::fs::read_to_string(file_path).map_err(|e| format!("Cannot read file: {}", e))?;
            let result = crate::extract::extract_entity(
                &source,
                entity_name,
                target_folder,
                entity_types,
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
        "rename_entity" => {
            let file_path = args.get("file_path").and_then(Value::as_str).ok_or("file_path is required")?;
            let old_name = args.get("old_name").and_then(Value::as_str).ok_or("old_name is required")?;
            let new_name = args.get("new_name").and_then(Value::as_str).ok_or("new_name is required")?;
            let changed = crate::rename_entity::rename_entity(file_path, old_name, new_name)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : format!("Renamed: {} (Changed: {})", file_path, changed) }] } }))
        }
        "fix_cargo_errors" => {
            let manifest_path = args.get("manifest_path").and_then(Value::as_str).ok_or("manifest_path is required")?;
            let result = crate::fix_cargo::fix_cargo_errors(manifest_path)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : result }] } }))
        }
        "optimize_imports" => {
            let file_path = args.get("file_path").and_then(Value::as_str).ok_or("file_path is required")?;
            let result = crate::optimize_imports::optimize_imports(file_path)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : result }] } }))
        }
        "ssr" => {
            let file_path = args.get("file_path").and_then(Value::as_str).ok_or("file_path is required")?;
            let pattern = args.get("pattern").and_then(Value::as_str).ok_or("pattern is required")?;
            let replacement = args.get("replacement").and_then(Value::as_str).ok_or("replacement is required")?;
            let changed = crate::ssr::ssr(file_path, pattern, replacement)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : format!("SSR on {}: {} -> {} (Changed: {})", file_path, pattern, replacement, changed) }] } }))
        }
        "expand_macros" => {
            let target = args.get("target").and_then(Value::as_str).ok_or("target is required")?;
            let result = crate::macro_expander::expand_macros(target)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : result }] } }))
        }
        "analyze_dependencies" => {
            let dir_path = args.get("dir_path").and_then(Value::as_str).ok_or("dir_path is required")?;
            let result = crate::dependency_graph_analyzer::analyze_dependencies(dir_path)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : result }] } }))
        }
        "preflight_validator" => {
            let manifest_path = args.get("manifest_path").and_then(Value::as_str).ok_or("manifest_path is required")?;
            let result = crate::preflight_validator::validate_project(manifest_path)?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : result }] } }))
        }
        "split_folder_entities" => {
            let dir_path = args.get("dir_path").and_then(Value::as_str).ok_or("dir_path is required")?;
            let generate_reexport = args.get("generate_reexport").and_then(Value::as_bool).unwrap_or(true);
            let entity_types = args.get("entity_types").and_then(Value::as_array).map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
            crate::split_file::split_folder_entities(dir_path, generate_reexport, entity_types).map_err(|e| e.to_string())?;
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : format!("Successfully split directory: {}", dir_path) }] } }))
        }
        "discover_multi_entity_files" => {
            let dir_path = args.get("dir_path").and_then(Value::as_str).ok_or("dir_path is required")?;
            let files = crate::split_file::discover_multi_entity_files(dir_path).map_err(|e| e.to_string())?;
            let result_map: HashMap<String, usize> = files.into_iter().map(|(k, v)| (k.to_string_lossy().to_string(), v)).collect();
            Ok(json!({ "jsonrpc" : "2.0", "id" : id, "result" : { "content" : [{ "type" : "text", "text" : serde_json::to_string_pretty(&result_map).unwrap() }] } }))
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
        // This test might fail now because I added many tools.
        // I need to adjust it to check for existence, not exact length.
        assert!(tools.len() >= 1);
        assert!(tools.iter().any(|t| t["name"] == "extract_entity"));
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
