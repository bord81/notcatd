use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use crate::log::*;
use crate::log_def::LogPriority;
use crate::msg_sink::LogMessage;
use crate::msg_sink::MessageSink;

pub struct LocalFileSink {
    pub log_file: Option<File>,
    local_file_sm: RotatingFileSink,
}

static LOG_DIR: &str = "/data/misc/notcat";
static LOG_FILE: &str = "notcat.log";
static MAX_LOG_FILES_SIZE: u64 = 100 * 1024 * 1024; // 100 MB
static MAX_LOG_FILES_COUNT: u32 = 5;
static MAX_LOG_FILE_SIZE: u64 = MAX_LOG_FILES_SIZE / MAX_LOG_FILES_COUNT as u64;

impl LocalFileSink {
    pub fn new() -> Self {
        LocalFileSink {
            log_file: None,
            local_file_sm: RotatingFileSink::new(),
        }
    }
}

impl MessageSink for LocalFileSink {
    fn init(&mut self) -> Result<(), String> {
        self.local_file_sm.handle_event(LoggingEvent::Init);
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
        self.local_file_sm
            .handle_event(LoggingEvent::SendMessage(msg));
    }

    fn close(&mut self) -> Result<(), String> {
        self.local_file_sm.handle_event(LoggingEvent::Close);
        Ok(())
    }
}

struct CurrentSinkFileData {
    pub number: u32,
    pub file: Option<File>,
    pub current_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoggingState {
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug)]
enum LoggingEvent {
    Init,
    SendMessage(String),
    Close,
}

struct RotatingFileSink {
    current_file_data: CurrentSinkFileData,
    state: LoggingState,
}

impl RotatingFileSink {
    fn new() -> Self {
        RotatingFileSink {
            current_file_data: CurrentSinkFileData {
                number: 0,
                file: None,
                current_size: 0,
            },
            state: LoggingState::Starting,
        }
    }

    fn handle_event(&mut self, event: LoggingEvent) {
        self.state = match (&self.state, event) {
            (LoggingState::Starting, LoggingEvent::Init) => {
                let mut return_state = LoggingState::Running;
                for i in (0..MAX_LOG_FILES_COUNT).rev() {
                    let file_path = PathBuf::from(format!("{}/{}.{}", LOG_DIR, LOG_FILE, i));
                    if file_path.exists() {
                        self.current_file_data.number = i;
                        let next_file = match OpenOptions::new().append(true).open(&file_path) {
                            Ok(file) => file,
                            Err(e) => {
                                loge!(LOG_TAG, "Failed to open log file: {}", e);
                                return_state = LoggingState::Error;
                                break;
                            }
                        };
                        self.current_file_data.file = Some(next_file);
                        self.current_file_data.current_size =
                            file_path.metadata().map(|m| m.len()).unwrap_or(0);
                        logv!(LOG_TAG, "Using existing log file: {}", file_path.display());
                        break;
                    } else if i == 0 {
                        self.current_file_data.number = 0;
                        self.current_file_data.current_size = 0;
                        let next_file = match OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(format!("{}/{}.0", LOG_DIR, LOG_FILE))
                        {
                            Ok(file) => file,
                            Err(e) => {
                                loge!(LOG_TAG, "Failed to create log file: {}", e);
                                return_state = LoggingState::Error;
                                break;
                            }
                        };
                        self.current_file_data.file = Some(next_file);
                        logv!(
                            LOG_TAG,
                            "Created new log file: {}",
                            format!("{}/{}.0", LOG_DIR, LOG_FILE)
                        );
                    }
                }
                return_state
            }
            (LoggingState::Running, LoggingEvent::SendMessage(msg)) => {
                let mut return_state = LoggingState::Running;
                if self.current_file_data.current_size + 1 + msg.len() as u64 >= MAX_LOG_FILE_SIZE {
                    logv!(LOG_TAG, "Rotating log file due to size limit.");
                    if self.current_file_data.number == MAX_LOG_FILES_COUNT - 1 {
                        if let Err(e) = std::fs::remove_file(PathBuf::from(format!(
                            "{}/{}.0",
                            LOG_DIR, LOG_FILE
                        ))) {
                            loge!(LOG_TAG, "Failed to remove oldest log file: {}", e);
                            return_state = LoggingState::Error;
                        } else {
                            for i in 1..MAX_LOG_FILES_COUNT {
                                let old_file_path =
                                    PathBuf::from(format!("{}/{}.{}", LOG_DIR, LOG_FILE, i));
                                let new_file_path =
                                    PathBuf::from(format!("{}/{}.{}", LOG_DIR, LOG_FILE, i - 1));
                                if let Err(e) = std::fs::rename(old_file_path, new_file_path) {
                                    loge!(LOG_TAG, "Failed to rotate log file: {}", e);
                                    return_state = LoggingState::Error;
                                    break;
                                }
                            }

                            let file_path = PathBuf::from(format!(
                                "{}/{}.{}",
                                LOG_DIR, LOG_FILE, self.current_file_data.number
                            ));
                            let next_file: Option<File> = match OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&file_path)
                            {
                                Ok(file) => Some(file),
                                Err(e) => {
                                    loge!(LOG_TAG, "Failed to open log file: {}", e);
                                    return_state = LoggingState::Error;
                                    None
                                }
                            };
                            self.current_file_data.file = next_file;
                            self.current_file_data.current_size = 0;
                        }
                    } else {
                        self.current_file_data.number += 1;
                        self.current_file_data.current_size = 0;
                        let file_path = PathBuf::from(format!(
                            "{}/{}.{}",
                            LOG_DIR, LOG_FILE, self.current_file_data.number
                        ));
                        let next_file: Option<File> = match OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(&file_path)
                        {
                            Ok(file) => Some(file),
                            Err(e) => {
                                loge!(LOG_TAG, "Failed to open log file: {}", e);
                                return_state = LoggingState::Error;
                                None
                            }
                        };
                        self.current_file_data.file = next_file;
                        logv!(LOG_TAG, "Created new log file: {}", file_path.display());
                    }
                }
                if return_state == LoggingState::Running && !self.current_file_data.file.is_none() {
                    self.current_file_data.current_size += 1 + msg.len() as u64;
                    if let Err(e) =
                        writeln!(self.current_file_data.file.as_mut().unwrap(), "{}", msg)
                    {
                        loge!(LOG_TAG, "Failed to write message: {}", e);
                        return_state = LoggingState::Error
                    }
                    if let Err(e) = self.current_file_data.file.as_mut().unwrap().flush() {
                        loge!(LOG_TAG, "Failed to flush log: {}", e);
                        return_state = LoggingState::Error
                    }
                }
                return_state
            }
            (LoggingState::Running, LoggingEvent::Close) => {
                if let Some(ref mut file) = self.current_file_data.file {
                    if let Err(e) = file.flush() {
                        loge!(LOG_TAG, "Failed to flush log: {}", e);
                    }
                }
                logv!(LOG_TAG, "Rotating file sink closed.");
                LoggingState::Stopping
            }
            _ => self.state,
        }
    }
}
