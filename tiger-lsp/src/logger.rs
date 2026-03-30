use log::{LevelFilter, Log};
use lsp_types::{LogMessageParams, MessageType};
use simplelog::SharedLogger;

use crate::{connection::Connection, notification::Notification};

#[derive(Debug)]
pub struct ConnectionLogger {
    level: LevelFilter,
    connection: Connection,
}

impl ConnectionLogger {
    pub fn new(level: LevelFilter) -> Box<Self> {
        Box::new(Self { level, connection: Connection::new() })
    }
}

impl Log for ConnectionLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let typ = match record.level() {
            log::Level::Error => MessageType::ERROR,
            log::Level::Warn => MessageType::WARNING,
            log::Level::Info => MessageType::INFO,
            log::Level::Debug | log::Level::Trace => MessageType::LOG,
        };

        let message = format!("{}: {}", record.target(), record.args());

        let params = LogMessageParams { typ, message };

        log::trace!("{}", serde_json::to_string(&params).unwrap());

        let _ = self.connection.send_notification(&Notification {
            method: "window/logMessage".into(),
            params: std::mem::take(serde_json::to_value(params).unwrap().as_object_mut().unwrap()),
        });
    }

    fn flush(&self) {}
}

impl SharedLogger for ConnectionLogger {
    fn level(&self) -> LevelFilter {
        self.level
    }

    fn config(&self) -> Option<&simplelog::Config> {
        None
    }

    fn as_log(self: Box<Self>) -> Box<dyn Log> {
        Box::new(*self)
    }
}
