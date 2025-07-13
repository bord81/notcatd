use crate::log::*;
use crate::log_def::LogPriority;
use crate::log_def::LogTimeStamp;
use crate::msg_sink::MessageSink;
use crate::msg_sink::SinkType;
use std::thread;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;

pub trait MessageProcessor<M, R, H> {
    fn run(sink_vec: Vec<M>, receiver: R) -> H;
}

pub struct OutputHandler;

impl MessageProcessor<SinkType, Receiver<Vec<u8>>, thread::JoinHandle<()>> for OutputHandler {
    fn run(mut sink_vec: Vec<SinkType>, mut receiver: Receiver<Vec<u8>>) -> thread::JoinHandle<()> {
        for sink in &mut sink_vec {
            if let Err(e) = sink.init() {
                loge!(LOG_TAG, "[OutputHandler] Sink init failed: {}", e);
            }
        }
        thread::spawn(move || {
            while let Some(data) = receiver.blocking_recv() {
                let msg = String::from_utf8_lossy(&data);
                logd!(LOG_TAG, "[OutputHandler] Got: {}", msg);
                // iterate over sink_vec and send the message to each sink
                for sink in &mut sink_vec {
                    sink.send_message(crate::log_def::LogMessage {
                        pid: 0,
                        priority: LogPriority::Debug,
                        tag: Some(LOG_TAG.to_string()),
                        message: msg.to_string(),
                        timestamp: LogTimeStamp {
                            year: 1970,
                            month: 1,
                            day: 1,
                            hour: 0,
                            minute: 0,
                            second: 0,
                            millisecond: 0,
                        },
                    });
                }
            }
            logd!(LOG_TAG, "[OutputHandler] Channel closed, exiting.");
        })
    }
}
