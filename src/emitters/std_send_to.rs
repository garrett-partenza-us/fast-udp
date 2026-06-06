use anyhow::{Result, bail};
use std::net::{SocketAddr, UdpSocket};

use crate::emitters::UdpEmitter;

pub struct StdSendToEmitter {
    socket: UdpSocket,
    target: SocketAddr
}

impl StdSendToEmitter {
    pub fn new(target: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        Ok(Self { socket, target })
    }
}

impl UdpEmitter for StdSendToEmitter {
    fn name(&self) -> &'static str {
        "std-send-to"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        for _ in 0..packets {
            let bytes = self.socket.send_to(payload, self.target)?;

            if bytes != payload.len() {
                bail!(
                    "partial UDP datagram send: sent {} bytes out of {}",
                    bytes,
                    payload.len()
                );
            }
        }

        Ok(())
    }
}
