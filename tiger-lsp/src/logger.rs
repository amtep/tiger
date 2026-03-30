use log::{LevelFilter, Log};
use lsp_types::{LogMessageParams, MessageType};
use simplelog::SharedLogger;

use crate::{connection::Connection, notification::NotificationToClient};

#[derive(Debug, Clone, Copy)]
pub struct ConnectionLogger {
    level: LevelFilter,
}

impl ConnectionLogger {
    pub fn new(level: LevelFilter) -> Box<Self> {
        Box::new(Self { level })
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

        let _ = Connection::send_notification(&NotificationToClient::new(
            "window/logMessage",
            Some(serde_json::to_value(&params).unwrap()),
        ));

        if record.level() <= log::Level::Warn {
            let _ = Connection::send_notification(&NotificationToClient::new(
                "window/showMessage",
                // # logMessageParams is reused instead of the proper showMessageParams
                // # which are identical
                Some(serde_json::to_value(&params).unwrap()),
            ));
        }
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
