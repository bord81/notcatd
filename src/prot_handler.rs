use crate::{log::*, log_def::*};
use std::collections::HashMap;
use std::convert::TryInto;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender as Sender;

#[derive(Error, Debug, PartialEq)]
pub enum ClientError {
    #[error("Incorrect magic number: {0}")]
    IncorrectMagic(u32),
    #[error("Incorrect version number: {0}")]
    IncorrectVersion(u8),
    #[error("Incorrect header size: {0}")]
    IncorrectHeaderSize(usize),
    #[error("Incorrect message size: {0}")]
    IncorrectMessageSize(usize),
    #[error("Internal error occurred")]
    InternalError,
}

#[allow(dead_code)]
pub struct ProtocolHandler {
    fds_pids: HashMap<i32, u32>,
    sender_channel: Sender<Vec<u8>>,
}

static CONN_MAGIC: u32 = 0xb05acafe;

static CURRENT_VERSION: u8 = 1;

static VERSION_1_HSH_SZ: usize = 9; // 4 bytes for magic, 1 byte for version, 4 bytes for pid
#[allow(dead_code)]
static VERSION_1_MSG_SZ: usize = 14; // 4 bytes for message size, 1 byte for priority, 9 bytes for timestamp

impl ProtocolHandler {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        ProtocolHandler {
            fds_pids: HashMap::new(),
            sender_channel: sender,
        }
    }

    pub fn process_buffer(&mut self, fd: i32, buffer: &[u8]) -> Result<(), ClientError> {
        let buffer_len = buffer.len();
        let mut buffer_ptr: usize = 0;
        loop {
            if buffer_ptr >= buffer_len {
                break Ok(());
            }
            if self.fds_pids.contains_key(&fd) {
                if buffer_len - buffer_ptr < 4 {
                    return Err(ClientError::IncorrectMessageSize(buffer_len - buffer_ptr));
                }
                let msg_size =
                    u32::from_be_bytes(buffer[buffer_ptr..buffer_ptr + 4].try_into().unwrap())
                        as usize;
                buffer_ptr += 4;
                if buffer_len - buffer_ptr < msg_size {
                    return Err(ClientError::IncorrectMessageSize(buffer_len - buffer_ptr));
                }
                if self
                    .sender_channel
                    .send(buffer[buffer_ptr..buffer_ptr + msg_size].to_vec())
                    .is_err()
                {
                    return Err(ClientError::InternalError);
                }
                buffer_ptr += msg_size;
            } else {
                if buffer.len() < VERSION_1_HSH_SZ {
                    return Err(ClientError::IncorrectHeaderSize(buffer.len()));
                }
                let magic = u32::from_be_bytes(buffer[0..4].try_into().unwrap());
                if magic != CONN_MAGIC {
                    return Err(ClientError::IncorrectMagic(magic));
                }
                let version = u8::from_be_bytes(buffer[4..5].try_into().unwrap());
                if version != CURRENT_VERSION {
                    return Err(ClientError::IncorrectVersion(version));
                }
                let pid = u32::from_be_bytes(buffer[5..9].try_into().unwrap());
                self.fds_pids.insert(fd, pid);
                logd!(
                    LOG_TAG,
                    "[ProtocolHandler] New connection: fd={}, pid={}",
                    fd,
                    pid
                );
                buffer_ptr += VERSION_1_HSH_SZ;
            }
        }
    }

    pub fn remove_fd(&mut self, fd: i32) {
        self.fds_pids.remove(&fd);
    }
}
