use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ResourceSchema {
    pub api_version: String,
    pub group: String,
    pub version: String,
    pub kind: String,
    pub namespaced: bool,
    pub schema: Value,
    pub definitions: serde_json::Map<String, Value>,
}
