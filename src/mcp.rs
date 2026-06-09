//! MCP stdio server — JSON-RPC transport over stdin/stdout.

use std::io::{BufRead, BufReader, Write};

use serde::Deserialize;
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

        // Send startup notification on stderr so it doesn't interfere with JSON-RPC protocol
        if request.method == "initialize" {
            eprintln!("rust-refactor-mcp server started");
        }

        let response = match request.method.as_str() {
            "initialize" => handle_initialize(&request.id),
            "initialized" => continue, // notification, no response
            "tools/list" => handle_tools_list(&request.id),
            "tools/call" => handle_tools_call(&request.id, request.params.as_ref().ok_or("Missing params for tools/call")?)?,
            _ => json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", request.method)
                }
            }),
        };

        let mut stdout = stdout.lock();
        writeln!(stdout, "{}", serde_json::to_string(&response).map_err(|e| e.to_string())?)
            .map_err(|e| format!("Write error: {}", e))?;
        stdout.flush().map_err(|e| format!("Flush error: {}", e))?;
    }

    Ok(())
}

fn handle_initialize(id: &Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "rust-refactor-mcp",
                "version": "0.1.0"
            }
        }
    })
}

fn handle_tools_list(id: &Option<Value>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "tools": [
                {
                    "name": "extract_entity",
                    "description": "Extract a named entity (struct, enum, trait, fn, impl) from a Rust source file into its own module file. Related impl blocks and test modules are extracted together. Cross-file use statements are updated automatically.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "Path to the Rust source file containing the entity"
                            },
                            "entity_name": {
                                "type": "string",
                                "description": "Name of the entity to extract"
                            },
                            "target_folder": {
                                "type": "string",
                                "description": "Directory where the new module file will be written"
                            },
                            "entity_type": {
                                "type": "string",
                                "description": "Hint for entity type: struct, enum, fn, trait, impl",
                                "enum": ["struct", "enum", "fn", "trait", "impl"]
                            }
                        },
                        "required": ["file_path", "entity_name", "target_folder"]
                    }
                }
            ]
        }
    })
}

fn handle_tools_call(id: &Option<Value>, params: &Value) -> Result<Value, String> {
    let args = params
        .get("arguments")
        .ok_or("Missing arguments")?;

    let file_path = args
        .get("file_path")
        .and_then(Value::as_str)
        .ok_or("file_path is required")?;

    let entity_name = args
        .get("entity_name")
        .and_then(Value::as_str)
        .ok_or("entity_name is required")?;

    let target_folder = args
        .get("target_folder")
        .and_then(Value::as_str)
        .ok_or("target_folder is required")?;

    let entity_type = args
        .get("entity_type")
        .and_then(Value::as_str);

    // Read source
    let source = std::fs::read_to_string(file_path)
        .map_err(|e| format!("Cannot read file: {}", e))?;

    // Extract — now handles source rewrite, import cleanup, parent mod, and usage files
    let result = crate::extract::extract_entity(
        &source, entity_name, target_folder, entity_type, Some(file_path),
    )?;

    Ok(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": format!(
                        "Extracted {} → {}\nItems: {}\nUsage files updated: {}",
                        entity_name,
                        result.new_file_path,
                        result.items_extracted.join(", "),
                        if result.usage_files_updated.is_empty() {
                            "none".to_string()
                        } else {
                            result.usage_files_updated.join(", ")
                        }
                    )
                }
            ],
            "structuredContent": {
                "new_file_path": result.new_file_path,
                "test_file_path": result.test_file_path,
                "items_extracted": result.items_extracted,
                "usage_files_updated": result.usage_files_updated,
                "source_updated": true
            }
        }
    }))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_response() {
        let resp = handle_initialize(&None);
        assert_eq!(resp["jsonrpc"], "2.0");
        assert_eq!(resp["result"]["serverInfo"]["name"], "rust-refactor-mcp");
        assert!(resp["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn tools_list() {
        let resp = handle_tools_list(&None);
        let tools = resp["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "extract_entity");
    }

    #[test]
    fn tools_call_missing_file_path() {
        let params = json!({
            "name": "extract_entity",
            "arguments": {
                "entity_name": "Foo",
                "target_folder": "."
            }
        });
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("file_path"));
    }

    #[test]
    fn tools_call_missing_entity_name() {
        let params = json!({
            "name": "extract_entity",
            "arguments": {
                "file_path": "test.rs",
                "target_folder": "."
            }
        });
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("entity_name"));
    }

    #[test]
    fn tools_call_missing_target_folder() {
        let params = json!({
            "name": "extract_entity",
            "arguments": {
                "file_path": "test.rs",
                "entity_name": "Foo"
            }
        });
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("target_folder"));
    }

    #[test]
    fn tools_call_missing_arguments() {
        let params = json!({
            "name": "extract_entity"
        });
        let err = handle_tools_call(&None, &params).unwrap_err();
        assert!(err.contains("Missing arguments"));
    }

    #[test]
    fn tools_call_invalid_file() {
        let params = json!({
            "name": "extract_entity",
            "arguments": {
                "file_path": "nonexistent.rs",
                "entity_name": "Foo",
                "target_folder": "."
            }
        });
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

        let params = json!({
            "name": "extract_entity",
            "arguments": {
                "file_path": file_path.to_string_lossy(),
                "entity_name": "Foo",
                "target_folder": tmp.to_string_lossy()
            }
        });

        let result = handle_tools_call(&None, &params).unwrap();
        assert!(result["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Foo"));
        assert!(result["result"]["structuredContent"]["new_file_path"].is_string());

        std::fs::remove_dir_all(&tmp).ok();
    }
}
