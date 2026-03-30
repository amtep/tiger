use std::io::{Read, Write, stdin, stdout};

use anyhow::{Result, bail};
use serde::Deserialize;

use crate::notification::{Notification, NotificationToClient};
use crate::request::Request;
use crate::response::Response;

#[derive(Debug)]
pub struct Connection {
    buffer: String,
    byte_buffer: Vec<u8>,
}

#[derive(Debug)]
pub enum Message {
    Request(Request),
    Notification(Notification),
}

impl Connection {
    pub fn new() -> Self {
        Self { buffer: String::new(), byte_buffer: Vec::new() }
    }

    pub fn message(&mut self) -> Result<Message> {
        let mut size = None;
        loop {
            self.buffer.clear();
            let bytes = stdin().read_line(&mut self.buffer)?;
            if bytes == 0 {
                bail!("stdin EOF");
            }
            if let Some(header) = self.buffer.strip_suffix("\r\n") {
                if let Some(size_str) = header.strip_prefix("Content-Length: ") {
                    size = Some(size_str.parse()?);
                }
                if header.is_empty() {
                    break;
                }
            } else {
                bail!("malformed header");
            }
        }

        if let Some(size) = size {
            self.byte_buffer.resize(size, 0);
            stdin().read_exact(&mut self.byte_buffer)?;
            let body = str::from_utf8(&self.byte_buffer)?;
            let body: serde_json::Map<String, serde_json::Value> = serde_json::from_str(body)?;
            let message = if body.contains_key("id") {
                Message::Request(Request::deserialize(&body)?)
            } else {
                Message::Notification(Notification::deserialize(&body)?)
            };
            Ok(message)
        } else {
            bail!("message with no content length");
        }
    }

    pub fn send_response(response: &Response) -> Result<()> {
        let body = serde_json::to_string(response)?;
        let response = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        stdout().write_all(response.as_bytes())?;
        stdout().flush()?;
        Ok(())
    }

    pub fn send_notification(notification: &NotificationToClient) -> Result<()> {
        let body = serde_json::to_string(notification)?;
        let response = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        stdout().write_all(response.as_bytes())?;
        stdout().flush()?;
        Ok(())
    }
}
