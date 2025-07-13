mod log;
mod log_def;
mod msg_proc;
mod msg_sink;
mod msg_srv;
#[allow(unused_imports)]
mod prot_handler;
use crate::log::*;
use crate::log_def::LogPriority;

use crate::msg_sink::SinkType;
use crate::msg_sink::SinkTypeOrdinal;
use crate::prot_handler::LogPacket;
use crate::prot_handler::ProtocolHandler;
use msg_proc::{MessageProcessor, OutputHandler};
use msg_srv::{EpollServer, MessageServer};

use tokio::task;

#[tokio::main]
async fn main() {
    logi!(LOG_TAG, "Daemon is starting");

    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<LogPacket>();

    let prot_handler = ProtocolHandler::new(tx.clone());

    let server_handle = match EpollServer::run(prot_handler) {
        Ok(handle) => handle,
        Err(e) => {
            loge!(LOG_TAG, "Error starting server: {}", e);
            return;
        }
    };

    let sink_vec = vec![
        SinkType::new(SinkTypeOrdinal::LocalFileType).unwrap(),
        SinkType::new(SinkTypeOrdinal::AndroidNativeType).unwrap(),
    ];

    let receiver_handle = OutputHandler::run(sink_vec, rx);

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
