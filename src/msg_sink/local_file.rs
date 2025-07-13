use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::log::*;
use crate::log_def::LogPriority;
use crate::msg_sink::LogMessage;
use crate::msg_sink::MessageSink;

pub struct LocalFileSink {
    pub log_file: Option<File>,
}

static LOG_DIR: &str = "/data/vendor/notcat";
static LOG_FILE: &str = "/data/vendor/notcat/notcat.log";

impl MessageSink for LocalFileSink {
    fn init(&mut self) -> Result<(), String> {
        if !Path::new(LOG_DIR).exists() {
            std::fs::create_dir_all(LOG_DIR)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        if !Path::new(LOG_FILE).exists() {
            File::create(LOG_FILE).map_err(|e| format!("Failed to create log file: {}", e))?;
        }
        let log_file_path = PathBuf::from(LOG_FILE);
        self.log_file = Some(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)
                .map_err(|e| format!("Failed to open log file: {}", e))?,
        );
        Ok(())
    }

    fn send_message(&mut self, message: LogMessage) {
        let priority_str = match message.priority {
            LogPriority::Verbose => "V",
            LogPriority::Debug => "D",
            LogPriority::Info => "I",
            LogPriority::Warn => "W",
            LogPriority::Error => "E",
            _ => "U", // Unknown
        };
        let msg = format!(
            "[{}] {} {}-{}-{} {}:{}:{}-{} {}",
            message.pid,
            priority_str,
            message.timestamp.year,
            message.timestamp.month,
            message.timestamp.day,
            message.timestamp.hour,
            message.timestamp.minute,
            message.timestamp.second,
            message.timestamp.millisecond,
            message.message
        );

        if self.log_file.is_none() {
            loge!(LOG_TAG, "[MessageSink] Log file is not initialized.");
            return;
        }

        let log_file = self.log_file.as_mut().unwrap();
        if let Err(e) = writeln!(log_file, "{}", msg) {
            loge!(LOG_TAG, "[MessageSink] Failed to write to log file: {}", e);
        }

        if let Err(e) = log_file.flush() {
            loge!(LOG_TAG, "[MessageSink] Failed to flush log: {}", e);
        }
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}
