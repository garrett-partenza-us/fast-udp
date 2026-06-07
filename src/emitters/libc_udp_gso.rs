use std::{
    net::{SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use anyhow::{Result, bail};

use crate::emitters::UdpEmitter;

const UDP_SEGMENT: libc::c_int = 103;
const MAX_GSO_SEGMENTS: usize = 64;
const MAX_UDP_PAYLOAD_BYTES: usize = 65_507;

pub struct LibcUdpGsoEmitter {
    socket: UdpSocket,
    segments_per_send: usize,
}

impl LibcUdpGsoEmitter {
    pub fn new(target: SocketAddr, batch_size: usize, segment_size: usize) -> Result<Self> {
        if segment_size == 0 {
            bail!("UDP GSO segment size must be greater than zero");
        }

        let segments_per_send = batch_size
            .max(1)
            .min(MAX_GSO_SEGMENTS)
            .min(MAX_UDP_PAYLOAD_BYTES / segment_size);

        if segments_per_send == 0 {
            bail!("UDP GSO segment size is too large: {segment_size}");
        }

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        let segment_size = segment_size as libc::c_int;
        let result = unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::IPPROTO_UDP,
                UDP_SEGMENT,
                (&segment_size as *const libc::c_int).cast(),
                std::mem::size_of::<libc::c_int>() as libc::socklen_t,
            )
        };

        if result != 0 {
            bail!(
                "setsockopt UDP_SEGMENT failed: {}",
                std::io::Error::last_os_error()
            );
        }

        Ok(Self {
            socket,
            segments_per_send,
        })
    }
}

impl UdpEmitter for LibcUdpGsoEmitter {
    fn name(&self) -> &'static str {
        "libc-udp-gso"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        let fd = self.socket.as_raw_fd();
        let mut remaining = packets;

        let mut gso_payload = vec![0_u8; payload.len() * self.segments_per_send];
        for chunk in gso_payload.chunks_exact_mut(payload.len()) {
            chunk.copy_from_slice(payload);
        }

        while remaining > 0 {
            let segments = remaining.min(self.segments_per_send as u64) as usize;
            let bytes_to_send = segments * payload.len();

            let sent = unsafe { libc::send(fd, gso_payload.as_ptr().cast(), bytes_to_send, 0) };

            if sent < 0 {
                bail!("UDP GSO send failed: {}", std::io::Error::last_os_error());
            }

            if sent as usize != bytes_to_send {
                bail!(
                    "partial UDP GSO send: sent {} bytes out of {}",
                    sent,
                    bytes_to_send
                );
            }

            remaining -= segments as u64;
        }

        Ok(())
    }
}
