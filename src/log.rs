use crate::log_def::*;
use crate::msg_sink::AndroidLog;

pub static LOG_TAG: &str = "NotCat";

static ANDROID_LOGGER: AndroidLog = AndroidLog {};

pub fn log(priority: LogPriority, tag: &str, msg: &str) {
    android_logger::log(priority, tag, msg);

    // Add other logging outputs here, if needed

    mod android_logger {
        use crate::log::ANDROID_LOGGER;
        use crate::log_def::*;
        use crate::msg_sink::MessageSink;
        pub fn log(priority: LogPriority, tag: &str, msg: &str) {
            let log_msg: LogMessage = LogMessage {
                hash: 0,
                priority,
                timestamp: LogTimeStamp {
                    year: 0,
                    month: 0,
                    day: 0,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    millisecond: 0,
                },
                tag: Some(tag.to_string()),
                message: msg.to_string(),
            };
            ANDROID_LOGGER.send_message(log_msg);
        }
    }
}

#[allow(unused_macros)]
macro_rules! logv {
    ($a:expr, $fmt:expr $(, $args:expr)+ $(,)?) => {
        log(LogPriority::Verbose, $a, format!($fmt $(, $args)+).as_str())
    };
    ($a:expr, $fmt:expr $(,)?) => {
        log(LogPriority::Verbose, $a, format!($fmt).as_str())
    };
}

#[allow(unused_macros)]
macro_rules! logd {
    ($a:expr, $fmt:expr $(, $args:expr)+ $(,)?) => {
        log(LogPriority::Debug, $a, format!($fmt $(, $args)+).as_str())
    };
    ($a:expr, $fmt:expr $(,)?) => {
        log(LogPriority::Debug, $a, format!($fmt).as_str())
    };
}

#[allow(unused_macros)]
macro_rules! logi {
    ($a:expr, $fmt:expr $(, $args:expr)+ $(,)?) => {
        log(LogPriority::Info, $a, format!($fmt $(, $args)+).as_str())
    };
    ($a:expr, $fmt:expr $(,)?) => {
        log(LogPriority::Info, $a, format!($fmt).as_str())
    };
}

#[allow(unused_macros)]
macro_rules! logw {
    ($a:expr, $fmt:expr $(, $args:expr)+ $(,)?) => {
        log(LogPriority::Warn, $a, format!($fmt $(, $args)+).as_str())
    };
    ($a:expr, $fmt:expr $(,)?) => {
        log(LogPriority::Warn, $a, format!($fmt).as_str())
    };
}

#[allow(unused_macros)]
macro_rules! loge {
    ($a:expr, $fmt:expr $(, $args:expr)+ $(,)?) => {
        log(LogPriority::Error, $a, format!($fmt $(, $args)+).as_str())
    };
    ($a:expr, $fmt:expr $(,)?) => {
        log(LogPriority::Error, $a, format!($fmt).as_str())
    };
}

#[allow(unused_macros)]
macro_rules! logf {
    ($a:expr, $fmt:expr $(, $args:expr)+ $(,)?) => {
        log(LogPriority::Fatal, $a, format!($fmt $(, $args)+).as_str())
    };
    ($a:expr, $fmt:expr $(,)?) => {
        log(LogPriority::Fatal, $a, format!($fmt).as_str())
    };
}

pub(crate) use logd;
pub(crate) use loge;
#[allow(unused_imports)]
pub(crate) use logf;
pub(crate) use logi;
pub(crate) use logv;
pub(crate) use logw;
