use crate::log::*;
use crate::log_def::LogPriority;
use std::thread;
use tokio::sync::mpsc::UnboundedReceiver as Receiver;

pub trait MessageProcessor<R, H> {
    fn run(receiver: R) -> H;
}

pub struct ChannelProcessor;

impl MessageProcessor<Receiver<Vec<u8>>, thread::JoinHandle<()>> for ChannelProcessor {
    fn run(mut receiver: Receiver<Vec<u8>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            while let Some(data) = receiver.blocking_recv() {
                logd!(
                    LOG_TAG,
                    "[ChannelProcessor] Got: {}",
                    String::from_utf8_lossy(&data)
                );
            }
            logd!(LOG_TAG, "[ChannelProcessor] Channel closed, exiting.");
        })
    }
}
