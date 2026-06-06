use anyhow::{Result, bail};
use std::net::{SocketAddr, UdpSocket};

use crate::emitters::UdpEmitter;

pub struct StdConnectedEmitter {
    socket: UdpSocket,
}

impl StdConnectedEmitter {
    pub fn new(target: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        Ok(Self { socket })
    }
}

impl UdpEmitter for StdConnectedEmitter {
    fn name(&self) -> &'static str {
        "std-connected"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        for _ in 0..packets {
            let bytes = self.socket.send(payload)?;

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
