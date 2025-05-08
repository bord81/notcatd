mod android_native;
use crate::log_def::*;
use crate::msg_sink::android_native::*;

pub trait MessageSink {
    type MessageType;
    fn init(&mut self) -> Result<(), String>;
    fn send_message(&self, message: Self::MessageType);
    fn close(&mut self) -> Result<(), String>;
}

pub struct AndroidLog;

impl MessageSink for AndroidLog {
    type MessageType = LogMessage;

    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn send_message(&self, message: LogMessage) {
        let android_priority = match message.priority {
            LogPriority::Debug => AndroidLogPriority::Debug,
            LogPriority::Info => AndroidLogPriority::Info,
            LogPriority::Warn => AndroidLogPriority::Warn,
            LogPriority::Error => AndroidLogPriority::Error,
            LogPriority::Fatal => AndroidLogPriority::Fatal,
            _ => AndroidLogPriority::Verbose,
        };
        android_native::log_android_native(
            android_priority,
            message.tag.as_deref().unwrap_or(""),
            &message.message,
        );
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}
