use serde_derive::Deserialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct Notification {
    pub method: String,
    #[serde(default)]
    pub params: Map<String, Value>,
}
