use serde_derive::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[allow(unused)] // Many variants are not constructed, but are here for documentation
#[allow(clippy::enum_variant_names)] // We didn't choose the names. They are from the protocol.
pub enum ErrorCode {
    ParseError = -32700,
    InvalidRequest = -32600,
    MethodNotFound = -32601,
    InvalidParams = -32602,
    InternalError = -32603,
    ServerNotInitialized = -32002,
    UnknownErrorCode = -32001,
    RequestFailed = -32802,
    ServerCancelled = -32803,
    ContentModified = -32801,
    RequestCancelled = -32800,
}
