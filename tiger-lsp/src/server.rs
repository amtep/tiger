use log::info;
use serde_json::{Map, Value};

use crate::error_codes::ErrorCode;
use crate::response::Response;

#[derive(Debug)]
pub struct Server {
    initialized: bool,
    shutdown: bool,
}

impl Server {
    pub fn new() -> Self {
        Self { initialized: false, shutdown: false }
    }

    pub fn initialize(&mut self, id: Value, params: &Map<String, Value>) -> Response {
        if let Some(info) = params.get("clientInfo") {
            let mut client_name = "UNKNOWN".to_owned();
            if let Some(name) = info.get("name") {
                client_name = name.to_string();
            }
            if let Some(version) = info.get("version") {
                client_name = format!("{client_name} {version}");
            }
            info!("initializing for {client_name}");
        } else {
            info!("initializing");
        }

        if let Some(capabilities) = params.get("capabilities") {
            if let Some(general) = capabilities.get("general") {
                if let Some(position_encoding) = general.get("positionEncoding") {
                    if let Some(position_encoding) = position_encoding.as_array() {
                        if !position_encoding.contains(&Value::String("utf-8".to_string())) {
                            let mut data: Map<String, Value> = Map::new();
                            data.insert("retry".to_string(), Value::Bool(false));
                            return Response::error(
                                id,
                                ErrorCode::InvalidParams,
                                "only utf-8 position encoding is supported",
                                Some(Value::Object(data)),
                            );
                        }
                    }
                }
            }
        }

        self.initialized = true;
        let mut result: Map<String, Value> = Map::new();

        let mut server_info: Map<String, Value> = Map::new();
        server_info.insert("name".to_string(), Value::String(env!("CARGO_PKG_NAME").to_string()));
        server_info
            .insert("version".to_string(), Value::String(env!("CARGO_PKG_VERSION").to_string()));
        result.insert("serverInfo".to_string(), Value::Object(server_info));

        let mut capabilities: Map<String, Value> = Map::new();
        capabilities.insert("positionEncoding".to_string(), Value::String("utf-8".to_string()));
        result.insert("capabilities".to_string(), Value::Object(capabilities));
        Response::result(id, Value::Object(result))
    }

    pub fn shutdown(&mut self, id: Value) -> Response {
        self.shutdown = true;
        Response::result(id, Value::Null)
    }
}
