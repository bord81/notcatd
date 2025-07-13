pub mod android_native;
pub mod local_file;
use crate::log_def::*;

pub trait MessageSink {
    fn init(&mut self) -> Result<(), String>;
    fn send_message(&mut self, message: LogMessage);
    fn close(&mut self) -> Result<(), String>;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum SinkTypeOrdinal {
    LocalFileType = 1,
    AndroidNativeType = 2,
}

pub enum SinkType {
    LocalFile {
        implem: local_file::LocalFileSink,
        ordinal: SinkTypeOrdinal,
    },
    AndroidNative {
        implem: android_native::AndroidLog,
        ordinal: SinkTypeOrdinal,
    },
}

impl MessageSink for SinkType {
    fn init(&mut self) -> Result<(), String> {
        match self {
            SinkType::LocalFile { implem, .. } => implem
                .init()
                .map_err(|e| format!("LocalFileSink init failed: {}", e)),
            SinkType::AndroidNative { implem: _i, .. } => Ok(()),
        }
    }

    fn send_message(&mut self, message: LogMessage) {
        match self {
            SinkType::LocalFile { implem, .. } => implem.send_message(message),
            SinkType::AndroidNative { implem, .. } => implem.send_message(message),
        }
    }

    fn close(&mut self) -> Result<(), String> {
        match self {
            _ => Ok(()),
        }
    }
}

impl SinkType {
    pub fn new(ordinal: SinkTypeOrdinal) -> Option<Self> {
        match ordinal {
            SinkTypeOrdinal::LocalFileType => Some(SinkType::LocalFile {
                implem: local_file::LocalFileSink { log_file: None },
                ordinal,
            }),
            SinkTypeOrdinal::AndroidNativeType => Some(SinkType::AndroidNative {
                implem: android_native::AndroidLog,
                ordinal,
            }),
        }
    }
    pub fn get_ordinal(&self) -> &SinkTypeOrdinal {
        match self {
            SinkType::LocalFile { ordinal, .. } => ordinal,
            SinkType::AndroidNative { ordinal, .. } => ordinal,
        }
    }
}
