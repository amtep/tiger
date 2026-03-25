use std::collections::HashMap;

use log::{info, trace};
use serde_json::{Map, Value, json};

use crate::error_codes::ErrorCode;
use crate::lsp_types::{DidChangeTextDocumentParams, DidOpenTextDocumentParams};
use crate::openfile::OpenFile;
use crate::response::Response;

#[derive(Debug)]
pub struct Server {
    initialized: bool,
    shutdown: bool,
    open: HashMap<String, OpenFile>,
}

impl Server {
    pub fn new() -> Self {
        Self { initialized: false, shutdown: false, open: HashMap::default() }
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

        if let Some(capabilities) = params.get("capabilities")
            && let Some(general) = capabilities.get("general")
            && let Some(position_encoding) = general.get("positionEncoding")
            && let Some(position_encoding) = position_encoding.as_array()
            && position_encoding.contains(&Value::String("utf-8".to_string()))
        {
            // do nothing, it's ok
        } else {
            let data = json!({
                "retry": false,
            });
            return Response::error(
                id,
                ErrorCode::InvalidParams,
                "only utf-8 position encoding is supported",
                Some(data),
            );
        }

        self.initialized = true;
        let result = json!({
            "serverInfo": {
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "positionEncoding": "utf-8",
            },
            "textDocumentSync": {
                "openClose": true,
                "change": 2,
            },
        });
        Response::result(id, result)
    }

    pub fn shutdown(&mut self, id: Value) -> Response {
        self.shutdown = true;
        Response::result(id, Value::Null)
    }

    pub fn did_open(&mut self, params: &Map<String, Value>) {
        if let Ok(did_open) =
            serde_json::from_value::<DidOpenTextDocumentParams>(Value::Object(params.clone()))
        {
            info!("open {}", &did_open.textDocument.uri);
            self.open.insert(did_open.textDocument.uri.clone(), did_open.textDocument.into());
        } else {
            trace!("could not parse didOpen");
        }
    }

    pub fn did_change(&mut self, params: &Map<String, Value>) {
        if let Ok(change) =
            serde_json::from_value::<DidChangeTextDocumentParams>(Value::Object(params.clone()))
        {
            if let Some(open_file) = self.open.get_mut(&change.textDocument.uri) {
                for change in &change.contentChanges {
                    open_file.apply_change(&change.range, &change.text);
                }
                open_file.version = change.textDocument.version;
            }
        } else {
            trace!("could not parse didChange");
        }
    }
}
