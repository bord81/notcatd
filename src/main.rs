#[allow(unused_imports)]
mod log_def;
mod log;
mod msg_proc;
mod msg_sink;
mod msg_srv;
use crate::log::*;
use crate::log_def::LogPriority;

use msg_srv::{EpollServer, MessageServer};
use msg_proc::{ChannelProcessor, MessageProcessor};

use tokio::task;

#[tokio::main]
async fn main() {
    logi!(LOG_TAG, "Daemon is starting");

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    let server_handle = match EpollServer::run(tx.clone()) {
        Ok(handle) => handle,
        Err(e) => {
            loge!(LOG_TAG, "Error starting server: {}", e);
            return;
        }   
    };
    let receiver_handle = ChannelProcessor::run(rx);

    match task::spawn_blocking(move || {
        server_handle.join().unwrap()?;
        receiver_handle.join().unwrap();
        Ok::<(), std::io::Error>(())
    }).await {
        Ok(Ok(_)) => {
            logi!(LOG_TAG, "Daemon is ");
        }
        Ok(Err(e)) => {
            loge!(LOG_TAG, "Error exiting: {}", e);
        }
        Err(e) => {
            loge!(LOG_TAG, "Task error exiting: {}", e);
        }
    }
}