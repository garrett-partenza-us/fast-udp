use anyhow::{Result, bail};
use std::net::{SocketAddr, UdpSocket};

use crate::emitters::UdpEmitter;

pub struct StdConnectedPinEmitter {
    socket: UdpSocket,
}

impl StdConnectedPinEmitter {
    pub fn new(target: SocketAddr, cpu: usize) -> Result<Self> {
        pin_current_thread_to_cpu(cpu)?;

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        Ok(Self { socket })
    }
}

impl UdpEmitter for StdConnectedPinEmitter {
    fn name(&self) -> &'static str {
        "std-connected-pin"
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

#[cfg(target_os = "linux")]
fn pin_current_thread_to_cpu(cpu: usize) -> Result<()> {
    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();

        libc::CPU_ZERO(&mut set);
        libc::CPU_SET(cpu, &mut set);

        let result = libc::sched_setaffinity(
            0,
            std::mem::size_of::<libc::cpu_set_t>(),
            &set,
        );

        if result != 0 {
            bail!(
                "sched_setaffinity failed: {}",
                std::io::Error::last_os_error()
            );
        }
    }

    Ok(())
}