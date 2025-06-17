use crate::log_def::*;
pub static LOG_TAG: &str = "NotCat";

pub fn log(priority: LogPriority, tag: &str, msg: &str) {
    android_logger::log(priority, tag, msg);

    // Add other logging outputs here, if needed

    mod android_logger {
        use crate::log_def::*;
        use crate::msg_sink::android_native::{convert_priority, log_android_native};
        pub fn log(priority: LogPriority, tag: &str, msg: &str) {
            let android_priority = convert_priority(priority);
            log_android_native(android_priority, tag, msg);
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
pub(crate) use logf;
pub(crate) use logi;
pub(crate) use logv;
pub(crate) use logw;
