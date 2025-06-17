#[allow(dead_code)]
#[repr(i32)]
pub enum AndroidLogPriority {
    Unknown = 0,
    Default = 1,
    Verbose = 2,
    Debug = 3,
    Info = 4,
    Warn = 5,
    Error = 6,
    Fatal = 7,
    Silent = 8,
}

extern "C" {
    fn __android_log_write(prio: i32, tag: *const i8, msg: *const i8) -> i32;
}

pub fn log_android_native(prio: AndroidLogPriority, tag: &str, msg: &str) {
    use std::ffi::CString;

    let tag_c = match CString::new(tag) {
        Ok(c) => c,
        Err(_) => return,
    };
    let msg_c = match CString::new(msg) {
        Ok(c) => c,
        Err(n) => {
            let nul_position = n.nul_position();
            if nul_position == 0 {
                return;
            }
            let mut valid_part = n.into_vec();
            valid_part.truncate(nul_position);
            unsafe {
                let valid_string = CString::from_vec_unchecked(valid_part);
                valid_string
            }
        }
    };

    unsafe {
        __android_log_write(prio as i32, tag_c.as_ptr(), msg_c.as_ptr());
    }
}

use crate::LogPriority;
use crate::msg_sink::LogMessage;
use crate::msg_sink::MessageSink;

pub struct AndroidLog;

pub fn convert_priority(priority: LogPriority) -> AndroidLogPriority {
    match priority {
        LogPriority::Debug => AndroidLogPriority::Debug,
        LogPriority::Info => AndroidLogPriority::Info,
        LogPriority::Warn => AndroidLogPriority::Warn,
        LogPriority::Error => AndroidLogPriority::Error,
        LogPriority::Fatal => AndroidLogPriority::Fatal,
        _ => AndroidLogPriority::Verbose,
    }
}

impl MessageSink for AndroidLog {
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn send_message(&mut self, message: LogMessage) {
        let android_priority = convert_priority(message.priority);
        log_android_native(
            android_priority,
            message.tag.as_deref().unwrap_or(""),
            &message.message,
        );
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}
