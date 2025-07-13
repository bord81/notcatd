use crate::{log::*, log_def::*, prot_handler::*};
use nix::{
    fcntl::{FcntlArg, FdFlag, OFlag, fcntl},
    sys::epoll::*,
    sys::socket::{MsgFlags, accept, listen, recv},
    unistd::close,
};
use rustutils::sockets::android_get_control_socket;
use std::{
    io,
    os::fd::AsFd,
    os::fd::BorrowedFd,
    os::unix::io::RawFd,
    thread::{self, JoinHandle},
};

pub trait MessageServer<L, H> {
    fn run(listener: L) -> io::Result<H>;
}

pub struct EpollServer;

static SOCKET_NAME: &str = "notcat_socket";
static MAX_CLIENTS_QUEUE: usize = 16;

struct FdWrapper(RawFd);

impl FdWrapper {
    pub fn new(fd: RawFd) -> Self {
        FdWrapper(fd)
    }
}

impl AsFd for FdWrapper {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.0) }
    }
}

impl MessageServer<ProtocolHandler, JoinHandle<io::Result<()>>> for EpollServer {
    fn run(mut prot_handler: ProtocolHandler) -> io::Result<JoinHandle<io::Result<()>>> {
        logv!(LOG_TAG, "[EpollServer] Starting...");

        let listener_fd = init_socket_fd()?;

        let epfd = init_epoll_fd(&listener_fd)?;

        let handle = thread::spawn(move || {
            let mut events = vec![EpollEvent::empty(); 16];
            logv!(LOG_TAG, "[EpollServer] Starting...OK");
            loop {
                // Wait for events
                let nfds = epoll_wait(epfd, &mut events, -1)?;
                for ev in &events[..nfds] {
                    let fd = ev.data() as RawFd;
                    if fd == listener_fd {
                        loop {
                            match accept(listener_fd) {
                                Ok(client_fd) => {
                                    logv!(
                                        LOG_TAG,
                                        "[EpollServer] Accepted new client connection: {}",
                                        client_fd
                                    );
                                    fcntl(client_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;
                                    let mut ev = EpollEvent::new(
                                        EpollFlags::EPOLLIN | EpollFlags::EPOLLET,
                                        client_fd as u64,
                                    );
                                    match epoll_ctl(epfd, EpollOp::EpollCtlAdd, client_fd, &mut ev)
                                    {
                                        Ok(_) => {
                                            logv!(
                                                LOG_TAG,
                                                "[EpollServer] Added client {}",
                                                client_fd
                                            );
                                        }
                                        Err(e) => {
                                            loge!(
                                                LOG_TAG,
                                                "[EpollServer] Error adding client to epoll: {}",
                                                e
                                            );
                                            break;
                                        }
                                    };
                                }
                                Err(nix::errno::Errno::EAGAIN) => break,
                                Err(e) => {
                                    loge!(LOG_TAG, "[EpollServer] Error accepting client: {}", e);
                                    break;
                                }
                            }
                        }
                    } else if ev.events().contains(EpollFlags::EPOLLIN) {
                        let mut buf = [0u8; 8096];
                        let mut input_buffer = Vec::with_capacity(8096);
                        loop {
                            match recv(fd, &mut buf, MsgFlags::MSG_DONTWAIT) {
                                Ok(0) => {
                                    logv!(LOG_TAG, "[EpollServer] Client {} disconnected", fd);
                                    epoll_ctl(epfd, EpollOp::EpollCtlDel, fd, None)?;
                                    prot_handler.remove_fd(fd);
                                    match close(fd) {
                                        Ok(_) => {
                                            logv!(LOG_TAG, "[EpollServer] Closed client {}", fd)
                                        }
                                        Err(e) => loge!(
                                            LOG_TAG,
                                            "[EpollServer] Error closing client {}: {}",
                                            fd,
                                            e
                                        ),
                                    }
                                    break;
                                }
                                Ok(n) => {
                                    input_buffer.extend_from_slice(&buf[..n]);
                                }
                                Err(nix::errno::Errno::EAGAIN) => {
                                    if let Err(e) = prot_handler.process_buffer(fd, &input_buffer) {
                                        loge!(
                                            LOG_TAG,
                                            "[EpollServer] Error processing buffer for client {}: {:?}",
                                            fd,
                                            e
                                        );
                                        match e {
                                            ClientError::InternalError => {
                                                break;
                                            }
                                            ClientError::IncorrectMessageSize(_) => {
                                                break;
                                            }
                                            _ => {
                                                epoll_ctl(epfd, EpollOp::EpollCtlDel, fd, None)?;
                                                match close(fd) {
                                                    Ok(_) => {
                                                        logv!(
                                                            LOG_TAG,
                                                            "[EpollServer] Closed client {}",
                                                            fd
                                                        )
                                                    }
                                                    Err(e) => loge!(
                                                        LOG_TAG,
                                                        "[EpollServer] Error closing client {}: {}",
                                                        fd,
                                                        e
                                                    ),
                                                }
                                            }
                                        }
                                    }
                                    break;
                                }
                                Err(e) => {
                                    loge!(
                                        LOG_TAG,
                                        "[EpollServer] Error reading from client {}: {}",
                                        fd,
                                        e
                                    );
                                    epoll_ctl(epfd, EpollOp::EpollCtlDel, fd, None)?;
                                    prot_handler.remove_fd(fd);
                                    match close(fd) {
                                        Ok(_) => {
                                            logv!(LOG_TAG, "[EpollServer] Closed client {}", fd)
                                        }
                                        Err(e) => loge!(
                                            LOG_TAG,
                                            "[EpollServer] Error closing client {}: {}",
                                            fd,
                                            e
                                        ),
                                    }
                                    break;
                                }
                            }
                        }
                    } else if ev
                        .events()
                        .contains(EpollFlags::EPOLLHUP | EpollFlags::EPOLLERR)
                    {
                        logw!(LOG_TAG, "[EpollServer] Client {} hung up or error", fd);
                        prot_handler.remove_fd(fd);
                        match epoll_ctl(epfd, EpollOp::EpollCtlDel, fd, None) {
                            Ok(_) => logv!(LOG_TAG, "[EpollServer] Removed client {}", fd),
                            Err(e) => {
                                loge!(LOG_TAG, "[EpollServer] Error removing client {}: {}", fd, e)
                            }
                        }
                        match close(fd) {
                            Ok(_) => logv!(LOG_TAG, "[EpollServer] Closed client {}", fd),
                            Err(e) => {
                                loge!(LOG_TAG, "[EpollServer] Error closing client {}: {}", fd, e)
                            }
                        }
                    }
                }
            }
        });

        Ok(handle)
    }
}

fn init_socket_fd() -> io::Result<RawFd> {
    let listener_fd =
        FdWrapper::new(android_get_control_socket(SOCKET_NAME).unwrap_or_else(|_| {
            logf!(LOG_TAG, "[EpollServer] Failed to create socket");
            -1
        }));

    match listen(&listener_fd, MAX_CLIENTS_QUEUE) {
        Ok(_) => {
            logv!(
                LOG_TAG,
                "[EpollServer] Listening on socket: {}",
                listener_fd.0
            );
        }
        Err(e) => {
            loge!(LOG_TAG, "[EpollServer] Error listening on socket: {}", e);
            return Err(e.into());
        }
    }
    match fcntl(listener_fd.0, FcntlArg::F_SETFD(FdFlag::FD_CLOEXEC)) {
        Ok(_) => (),
        Err(e) => {
            loge!(
                LOG_TAG,
                "[EpollServer] Error setting listener fd to non-blocking: {}",
                e
            );
            return Err(e.into());
        }
    }

    match fcntl(listener_fd.0, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)) {
        Ok(_) => {
            return Ok(listener_fd.0);
        }
        Err(e) => {
            loge!(
                LOG_TAG,
                "[EpollServer] Error setting listener fd to non-blocking: {}",
                e
            );
            return Err(e.into());
        }
    }
}

fn init_epoll_fd(fd: &RawFd) -> io::Result<RawFd> {
    let epfd = match epoll_create1(EpollCreateFlags::empty()) {
        Ok(fd) => fd,
        Err(e) => {
            logf!(LOG_TAG, "[EpollServer] Error creating epoll fd: {}", e);
            return Err(e.into());
        }
    };
    let mut event = EpollEvent::new(EpollFlags::EPOLLIN | EpollFlags::EPOLLET, *fd as u64);
    match epoll_ctl(epfd, EpollOp::EpollCtlAdd, *fd, &mut event) {
        Ok(_) => Ok(epfd),
        Err(e) => {
            logf!(
                LOG_TAG,
                "[EpollServer] Error adding listener fd to epoll: {}",
                e
            );
            return Err(e.into());
        }
    }
}
