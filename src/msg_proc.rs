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
                    let client_priority = match data.priority {
                        0 => LogPriority::Verbose,
                        1 => LogPriority::Debug,
                        2 => LogPriority::Info,
                        3 => LogPriority::Warn,
                        4 => LogPriority::Error,
                        _ => LogPriority::Verbose,
                    };
                    sink.send_message(crate::log_def::LogMessage {
                        pid: data.pid,
                        priority: client_priority,
                        message: String::from_utf8_lossy(&data.message).to_string(),
                        timestamp: LogTimeStamp {
                            year: u16::from_be_bytes([data.timestamp[0], data.timestamp[1]]),
                            month: data.timestamp[2],
                            day: data.timestamp[3],
                            hour: data.timestamp[4],
                            minute: data.timestamp[5],
                            second: data.timestamp[6],
                            millisecond: u16::from_be_bytes([data.timestamp[7], data.timestamp[8]]),
                        },
                    });
                }
            }
            logd!(LOG_TAG, "[OutputHandler] Channel closed, exiting.");
        })
    }
}
