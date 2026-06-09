use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
pub struct JsonRpcRequest {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
    #[serde(default)]
    pub id: Option<Value>,
    #[serde(default)]
    pub params: Option<Value>,
}
