use serde_derive::Serialize;
use serde_json::Value;

use crate::error_codes::ErrorCode;

#[derive(Debug, Clone, Serialize)]
pub struct Response {
    jsonrpc: &'static str,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    code: ErrorCode,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

impl Response {
    pub fn result(id: Value, result: Value) -> Self {
        Self { jsonrpc: "2.0", id, result: Some(result), error: None }
    }

    pub fn error<S: Into<String>>(
        id: Value,
        code: ErrorCode,
        message: S,
        data: Option<Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(ErrorResponse { code, message: message.into(), data }),
        }
    }
}
