#[allow(dead_code)]
#[repr(u8)]
pub enum LogPriority {
    Verbose,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}
#[allow(dead_code)]
pub struct LogTimeStamp {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    pub millisecond: u16,
}
#[allow(dead_code)]
pub struct LogMessage {
    pub hash: u64,
    pub priority: LogPriority,
    pub timestamp: LogTimeStamp,
    pub tag: Option<String>,
    pub message: String,
}
