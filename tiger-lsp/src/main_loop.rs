use anyhow::Result;
use log::info;

use crate::connection::{Connection, Message};
use crate::error_codes::ErrorCode;
use crate::response::Response;
use crate::server::Server;

pub fn main_loop() -> Result<()> {
    let mut connection = Connection::new();
    let mut server = Server::new();
    loop {
        match connection.message()? {
            Message::Request(request) => match &*request.method {
                "initialize" => {
                    connection.response(&server.initialize(request.id, &request.params))?;
                }
                "shutdown" => {
                    connection.response(&server.shutdown(request.id))?;
                }
                _ => {
                    info!("rejecting {} request", request.method);
                    connection.response(&Response::error(
                        request.id,
                        ErrorCode::MethodNotFound,
                        "method not found",
                        None,
                    ))?;
                }
            },
            Message::Notification(notification) => match &*notification.method {
                "exit" => {
                    info!("exiting in response to notification");
                    return Ok(());
                }
                "textDocument/didOpen" => {
                    server.did_open(&notification.params);
                }
                "textDocument/didChange" => {
                    server.did_change(&notification.params);
                }
                "textDocument/didClose" => {
                    server.did_close(&notification.params);
                }
                "workspace/didChangeConfiguration" => {
                    server.change_config(&notification.params);
                }
                _ => {
                    info!("ignoring {} notification", notification.method);
                }
            },
        }
    }
}
