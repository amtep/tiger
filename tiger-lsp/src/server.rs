use log::{error, info, trace, warn};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    HoverParams, HoverProviderCapability, InitializeParams, InitializeResult, PositionEncodingKind,
    ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextDocumentSyncOptions, Uri,
};
use partially::Partial;
use serde::Deserialize;
use serde_json::{Map, Value, json};

use crate::config::{Config, PartialConfig};
use crate::datatype_tables::DatatypeTables;
use crate::error_codes::ErrorCode;
use crate::game_concepts::GameConcepts;
use crate::hover::hover_description;
use crate::openfile::OpenFile;
use crate::response::Response;
use crate::util::{ClientToServer, HashMap, ServerToClient};

#[derive(Debug)]
pub struct Server {
    initialized: bool,
    shutdown: bool,
    utf16: bool,
    open: HashMap<Uri, OpenFile>,
    config: Config,
    datatype_tables: DatatypeTables,
    workspace_dir: Option<Uri>,
    game_concepts: GameConcepts,
}

impl Server {
    pub fn new() -> Self {
        Self {
            initialized: false,
            shutdown: false,
            utf16: true,
            open: HashMap::default(),
            config: Config::default(),
            workspace_dir: None,
            datatype_tables: DatatypeTables::new(),
            game_concepts: GameConcepts::new(),
        }
    }

    pub fn initialize(&mut self, id: Value, params: &Map<String, Value>) -> Response {
        let mut result = InitializeResult {
            server_info: Some(ServerInfo {
                name: env!("CARGO_PKG_NAME").to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    },
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
        };

        if let Ok(client) = InitializeParams::deserialize(params) {
            if let Some(info) = client.client_info {
                let client_info = if let Some(version) = info.version {
                    format!("{} {version}", info.name)
                } else {
                    info.name.clone()
                };
                info!("initializing for {client_info}");
            } else {
                info!("initializing");
            }

            self.utf16 = true; // the default
            if let Some(general) = client.capabilities.general
                && let Some(position_encodings) = general.position_encodings
            {
                if position_encodings.contains(&PositionEncodingKind::UTF8) {
                    self.utf16 = false;
                } else if !position_encodings.contains(&PositionEncodingKind::UTF16) {
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
            result.capabilities.position_encoding = Some(if self.utf16 {
                PositionEncodingKind::UTF16
            } else {
                PositionEncodingKind::UTF8
            });

            if let Some(init_options) = client.initialization_options {
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

            if let Some(game_dir) = self.config.game_dir() {
                match GameConcepts::load_game(game_dir) {
                    Ok(game_concepts) => {
                        self.game_concepts = game_concepts;
                    }
                    Err(err) => {
                        warn!("failed to load game concepts: {err}");
                    }
                }
            }

            if let Some(workspace_dirs) = client.workspace_folders {
                self.workspace_dir = workspace_dirs.first().map(|w| w.uri.clone());

                if let Some(workspace_dir) = &self.workspace_dir
                    && workspace_dir.scheme().map(|s| s.as_str()) == Some("file")
                {
                    let workspace_dir_path = workspace_dir.path().as_str();

                    if let Err(err) =
                        self.game_concepts.load_mod(std::path::Path::new(workspace_dir_path))
                    {
                        warn!("failed to load mod game concepts: {err}");
                    }
                }
            }

            self.initialized = true;
            // SAFETY: the type system enforces that we built a valid result
            Response::result(id, serde_json::to_value(&result).expect("server initialize result"))
        } else {
            let data = json!({
                "retry": false,
            });
            Response::error(
                id,
                ErrorCode::InvalidParams,
                "could not parse initialize params",
                Some(data),
            )
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
                        &self.game_concepts,
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
