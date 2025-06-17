pub mod android_native;
pub mod local_file;
use crate::log_def::*;

pub trait MessageSink {
    fn init(&mut self) -> Result<(), String>;
    fn send_message(&mut self, message: LogMessage);
    fn close(&mut self) -> Result<(), String>;
}

#[allow(dead_code)]
pub enum SinkType {
    LocalFile { implem: local_file::LocalFileSink },
    AndroidNative { implem: android_native::AndroidLog },
}

impl MessageSink for SinkType {
    fn init(&mut self) -> Result<(), String> {
        match self {
            SinkType::LocalFile { implem } => implem
                .init()
                .map_err(|e| format!("LocalFileSink init failed: {}", e)),
            SinkType::AndroidNative { implem: _i } => Ok(()),
        }
    }

    fn send_message(&mut self, message: LogMessage) {
        match self {
            SinkType::LocalFile { implem } => implem.send_message(message),
            SinkType::AndroidNative { implem } => implem.send_message(message),
        }
    }

    fn close(&mut self) -> Result<(), String> {
        match self {
            _ => Ok(()),
        }
    }
}
