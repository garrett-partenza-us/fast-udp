use std::{
    net::{SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use anyhow::{Result, bail};

use crate::emitters::UdpEmitter;

pub struct LibcSendEmitter {
    socket: UdpSocket,
}

impl LibcSendEmitter {
    pub fn new(target: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        Ok(Self { socket })
    }
}

impl UdpEmitter for LibcSendEmitter {
    fn name(&self) -> &'static str {
        "libc-send"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        let fd = self.socket.as_raw_fd();

        for _ in 0..packets {
            let sent = unsafe { libc::send(fd, payload.as_ptr().cast(), payload.len(), 0) };

            if sent < 0 {
                bail!("send failed: {}", std::io::Error::last_os_error());
            }

            if sent as usize != payload.len() {
                bail!(
                    "partial UDP datagram send: sent {} bytes out of {}",
                    sent,
                    payload.len()
                );
            }
        }

        Ok(())
    }
}
