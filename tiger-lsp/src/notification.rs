use serde_derive::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct Notification {
    pub method: String,
    #[serde(default)]
    pub params: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NotificationToClient {
    pub jsonrpc: &'static str,
    pub method: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl NotificationToClient {
    pub fn new(method: &'static str, params: Option<Value>) -> Self {
        Self { jsonrpc: "2.0", method, params }
    }
}
