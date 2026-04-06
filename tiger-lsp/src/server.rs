use log::{error, info, trace, warn};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    HoverParams, Position, Range, SemanticTokensParams, SemanticTokensRangeParams, Uri,
};
use partially::Partial;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::config::{Config, PartialConfig};
use crate::datatype_tables::DatatypeTables;
use crate::error_codes::ErrorCode;
use crate::hover::hover_description;
use crate::openfile::OpenFile;
use crate::response::Response;
use crate::semantic::SemanticTokens;
use crate::util::{ClientToServer, HashMap, ServerToClient};

#[derive(Debug)]
pub struct Server {
    initialized: bool,
    shutdown: bool,
    utf16: bool,
    open: HashMap<Uri, OpenFile>,
    config: Config,
    datatype_tables: DatatypeTables,
    semantic: SemanticTokens,
}

impl Server {
    pub fn new() -> Self {
        Self {
            initialized: false,
            shutdown: false,
            utf16: true,
            open: HashMap::default(),
            config: Config::default(),
            datatype_tables: DatatypeTables::new(),
            semantic: SemanticTokens::default(),
        }
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

        self.utf16 = true; // the default
        // If client types and modifiers are not in the initialization message,
        // then the client does not support semantic tokens so it's ok to have
        // an empty vector.
        let mut token_types = Vec::new();
        let mut token_modifiers = Vec::new();
        let mut client_types = Vec::new();
        let mut client_modifiers = Vec::new();

        if let Some(capabilities) = params.get("capabilities") {
            if let Some(general) = capabilities.get("general")
                && let Some(position_encoding) = general.get("positionEncodings")
                && let Some(position_encoding) = position_encoding.as_array()
            {
                if position_encoding.contains(&json!("utf-8")) {
                    self.utf16 = false;
                } else if !position_encoding.contains(&json!("utf-16")) {
                    let data = json!({
                        "retry": false,
                    });
                    return Response::error(
                        id,
                        ErrorCode::InvalidParams,
                        "only utf-8 or utf-16 position encodings are supported",
                        Some(data),
                    );
                }
            }

            if let Some(text_document) = capabilities.get("textDocument")
                && let Some(semantic_tokens) = text_document.get("semanticTokens")
            {
                if let Some(types) = semantic_tokens.get("tokenTypes")
                    && let Ok(types) = Vec::<&str>::deserialize(types)
                {
                    client_types = types;
                }

                if let Some(modifiers) = semantic_tokens.get("tokenModifiers")
                    && let Ok(modifiers) = Vec::<&str>::deserialize(modifiers)
                {
                    client_modifiers = modifiers;
                }

                (token_types, token_modifiers) =
                    self.semantic.initialize(&client_types, &client_modifiers);
            }
        }

        if let Some(init_options) = params.get("initializationOptions") {
            match PartialConfig::deserialize(init_options) {
                Ok(partial_config) => {
                    if self.config.apply_some(partial_config) {
                        trace!("initial config: {:?}", self.config);
                    }
                }
                Err(err) => {
                    warn!("failed to parse init options: {err}");
                }
            }
        }

        self.initialized = true;
        let result = json!({
            "serverInfo": {
                "name": env!("CARGO_PKG_NAME"),
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "positionEncoding": if self.utf16 { "utf-16" } else { "utf-8" },
                "hoverProvider": true,
                "semanticTokensProvider": {
                    "legend": {
                        "tokenTypes": token_types,
                        "tokenModifiers": token_modifiers,
                    },
                    "range": true,
                    "full": {
                        "delta": false,
                    },
                },
                "textDocumentSync": {
                    "openClose": true,
                    "change": 2,
                },
            },
        });
        Response::result(id, result)
    }

    pub fn semantic_tokens_full(&mut self, id: Value, params: &Map<String, Value>) -> Response {
        if let Ok(params) = SemanticTokensParams::deserialize(params) {
            if let Some(open) = self.open.get(&params.text_document.uri) {
                if open.language_id() != "pdx-localization" {
                    return Response::result(id, Value::Null);
                }
                let lines = open.get_lines();
                let range = Range {
                    start: Position { line: 0, character: 0 },
                    end: Position {
                        line: u32::try_from(open.num_lines()).expect("2^32 lines"),
                        character: 0,
                    },
                };
                let v = self.semantic.semantic_tokens_loca(self.utf16, &open.text, range, lines);
                Response::result(
                    id,
                    json!({
                        "data": v,
                    }),
                )
            } else {
                error!("semantic tokens request for non open file");
                Response::error(id, ErrorCode::InvalidRequest, "file not open", None)
            }
        } else {
            error!("could not parse semantic tokens request");
            Response::error(id, ErrorCode::InvalidParams, "could not parse params", None)
        }
    }

    pub fn semantic_tokens_range(&mut self, id: Value, params: &Map<String, Value>) -> Response {
        if let Ok(params) = SemanticTokensRangeParams::deserialize(params) {
            if let Some(open) = self.open.get(&params.text_document.uri) {
                if open.language_id() != "pdx-localization" {
                    return Response::result(id, Value::Null);
                }
                let start = params.range.start.line as usize;
                let end = if params.range.end.character > 0 {
                    params.range.end.line + 1
                } else {
                    params.range.end.line
                } as usize;
                if start > open.num_lines() || end > open.num_lines() {
                    return Response::result(id, Value::Null);
                }
                let lines = open.get_line_range(start..end);
                let v =
                    self.semantic.semantic_tokens_loca(self.utf16, &open.text, params.range, lines);
                Response::result(
                    id,
                    json!({
                        "data": v,
                    }),
                )
            } else {
                error!("semantic tokens request for non open file");
                Response::error(id, ErrorCode::InvalidRequest, "file not open", None)
            }
        } else {
            error!("could not parse semantic tokens request");
            Response::error(id, ErrorCode::InvalidParams, "could not parse params", None)
        }
    }

    pub fn shutdown(&mut self, id: Value) -> Response {
        self.shutdown = true;
        Response::result(id, Value::Null)
    }

    pub fn hover(&mut self, id: Value, params: &Map<String, Value>) -> Response {
        if let Ok(hover) = HoverParams::deserialize(params) {
            if let Some(open) =
                self.open.get(&hover.text_document_position_params.text_document.uri)
            {
                if open.language_id() != "pdx-localization" {
                    return Response::result(id, Value::Null);
                }
                let position = hover
                    .text_document_position_params
                    .position
                    .into_server(self.utf16, &open.text);
                if let Some(line) = open.get_line_around(position) {
                    let cursor = position.character;
                    let line_nr = position.line;
                    if let Some((contents, span)) = hover_description(
                        self.config.game,
                        &self.datatype_tables,
                        &line,
                        cursor as usize,
                    ) {
                        Response::result(
                            id,
                            json!({
                                "contents": {
                                    "kind": "markdown",
                                    "value": contents,
                                },
                                "range": span.into_range(line_nr).into_client(self.utf16, &open.text),
                            }),
                        )
                    } else {
                        Response::result(id, Value::Null)
                    }
                } else {
                    error!("hover request for invalid position");
                    Response::error(id, ErrorCode::InvalidRequest, "invalid position", None)
                }
            } else {
                error!("hover request for non open file");
                Response::error(id, ErrorCode::InvalidRequest, "file not open", None)
            }
        } else {
            error!("could not parse hover request");
            Response::error(id, ErrorCode::InvalidParams, "could not parse params", None)
        }
    }

    pub fn did_open(&mut self, params: &Map<String, Value>) {
        if let Ok(did_open) = DidOpenTextDocumentParams::deserialize(params) {
            info!("open {}", &did_open.text_document.uri.to_string());
            self.open.insert(did_open.text_document.uri.clone(), did_open.text_document.into());
        } else {
            error!("could not parse didOpen");
        }
    }

    pub fn did_change(&mut self, params: &Map<String, Value>) {
        if let Ok(change) = DidChangeTextDocumentParams::deserialize(params) {
            if let Some(open_file) = self.open.get_mut(&change.text_document.uri) {
                for change in &change.content_changes {
                    if let Some(range) = change.range {
                        let range = range.into_server(self.utf16, &open_file.text);
                        open_file.apply_change(range, &change.text);
                    } else {
                        open_file.new_text(&change.text);
                    }
                }
                open_file.version = change.text_document.version;
            }
        } else {
            error!("could not parse didChange");
        }
    }

    pub fn did_close(&mut self, params: &Map<String, Value>) {
        if let Ok(did_close) = DidCloseTextDocumentParams::deserialize(params) {
            info!("close {}", &did_close.text_document.uri.to_string());
            self.open.remove(&did_close.text_document.uri);
        } else {
            error!("could not parse didClose");
        }
    }

    pub fn change_config(&mut self, params: &Map<String, Value>) {
        let Some(settings) = params.get("settings") else {
            error!("could not parse didChangeConfiguration");
            return;
        };

        match PartialConfig::deserialize(settings) {
            Ok(partial_config) => {
                if self.config.apply_some(partial_config) {
                    trace!("new config: {:?}", self.config);
                }
            }
            Err(err) => {
                warn!("failed to parse init options: {err}");
            }
        }
    }
}
