use std::io::{Read, Write, stdin, stdout};

use anyhow::{Result, bail};
use log::trace;

use crate::notification::Notification;
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
            trace!("received\n{body}");
            let request = serde_json::from_str(body)
                .map(Message::Request)
                .or_else(|_| serde_json::from_str(body).map(Message::Notification))?;
            self.byte_buffer.clear();
            Ok(request)
        } else {
            bail!("message with no content length");
        }
    }

    #[allow(clippy::unused_self)] // self might be needed in the API in the future
    pub fn response(&mut self, response: &Response) -> Result<()> {
        let body = serde_json::to_string(response)?;
        let response = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        trace!("respond\n{body}");
        stdout().write_all(response.as_bytes())?;
        stdout().flush()?;
        Ok(())
    }
}
