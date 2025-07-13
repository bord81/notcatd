use crate::LogPacket;
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

impl MessageProcessor<SinkType, Receiver<LogPacket>, thread::JoinHandle<()>> for OutputHandler {
    fn run(
        mut sink_vec: Vec<SinkType>,
        mut receiver: Receiver<LogPacket>,
    ) -> thread::JoinHandle<()> {
        for sink in &mut sink_vec {
            if let Err(e) = sink.init() {
                loge!(LOG_TAG, "[OutputHandler] Sink init failed: {}", e);
            }
        }
        thread::spawn(move || {
            while let Some(data) = receiver.blocking_recv() {
                let sink_type = data.sink_type;
                // iterate over sink_vec and send the message to each sink
                for sink in &mut sink_vec {
                    if sink_type & (*sink.get_ordinal() as u8) == 0 {
                        continue;
                    }
                    sink.send_message(crate::log_def::LogMessage {
                        pid: data.pid,
                        priority: LogPriority::Debug,
                        message: String::from_utf8_lossy(&data.message).to_string(),
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
