mod log;
#[allow(unused_imports)]
mod log_def;
mod msg_proc;
mod msg_sink;
mod msg_srv;
use crate::log::*;
use crate::log_def::LogPriority;

use crate::msg_sink::SinkType;
use crate::msg_sink::android_native::AndroidLog;
use crate::msg_sink::local_file::LocalFileSink;
use msg_proc::{ChannelProcessor, MessageProcessor};
use msg_srv::{EpollServer, MessageServer};

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

    // The order of sinks matters; the ChannelProcessor uses this for routing.
    let sink_vec = vec![
        SinkType::LocalFile {
            implem: LocalFileSink { log_file: None },
        },
        SinkType::AndroidNative { implem: AndroidLog },
    ];

    let receiver_handle = ChannelProcessor::run(sink_vec, rx);

    match task::spawn_blocking(move || {
        server_handle.join().unwrap()?;
        receiver_handle.join().unwrap();
        Ok::<(), std::io::Error>(())
    })
    .await
    {
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
