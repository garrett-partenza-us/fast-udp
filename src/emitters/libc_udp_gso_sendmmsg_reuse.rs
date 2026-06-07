use std::{
    net::{SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use anyhow::{Result, bail};

use crate::emitters::UdpEmitter;

const UDP_SEGMENT: libc::c_int = 103;
const MAX_GSO_SEGMENTS: usize = 64;
const MAX_UDP_PAYLOAD_BYTES: usize = 65_507;

pub struct LibcUdpGsoSendmmsgReuseEmitter {
    socket: UdpSocket,
    segments_per_message: usize,
    datagrams_per_batch: usize,
    gso_payload: Vec<u8>,
    iovecs: Vec<libc::iovec>,
    messages: Vec<libc::mmsghdr>,
}

impl LibcUdpGsoSendmmsgReuseEmitter {
    pub fn new(target: SocketAddr, batch_size: usize, segment_size: usize) -> Result<Self> {
        if segment_size == 0 {
            bail!("UDP GSO segment size must be greater than zero");
        }

        let segments_per_message = batch_size
            .max(1)
            .min(MAX_GSO_SEGMENTS)
            .min(MAX_UDP_PAYLOAD_BYTES / segment_size);

        if segments_per_message == 0 {
            bail!("UDP GSO segment size is too large: {segment_size}");
        }

        let datagrams_per_batch = batch_size.max(1);
        let messages_per_batch = datagrams_per_batch.div_ceil(segments_per_message);
        let mut gso_payload = vec![0_u8; segment_size * segments_per_message];

        let mut iovecs = Vec::with_capacity(messages_per_batch);
        let mut messages = Vec::with_capacity(messages_per_batch);

        for _ in 0..messages_per_batch {
            iovecs.push(libc::iovec {
                iov_base: gso_payload.as_mut_ptr().cast(),
                iov_len: gso_payload.len(),
            });

            messages.push(libc::mmsghdr {
                msg_hdr: libc::msghdr {
                    msg_name: std::ptr::null_mut(),
                    msg_namelen: 0,
                    msg_iov: std::ptr::null_mut(),
                    msg_iovlen: 1,
                    msg_control: std::ptr::null_mut(),
                    msg_controllen: 0,
                    msg_flags: 0,
                },
                msg_len: 0,
            });
        }

        for index in 0..messages_per_batch {
            messages[index].msg_hdr.msg_iov = &mut iovecs[index];
        }

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        let segment_size_opt = segment_size as libc::c_int;
        let result = unsafe {
            libc::setsockopt(
                socket.as_raw_fd(),
                libc::IPPROTO_UDP,
                UDP_SEGMENT,
                (&segment_size_opt as *const libc::c_int).cast(),
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
            segments_per_message,
            datagrams_per_batch,
            gso_payload,
            iovecs,
            messages,
        })
    }
}

impl UdpEmitter for LibcUdpGsoSendmmsgReuseEmitter {
    fn name(&self) -> &'static str {
        "libc-udp-gso-sendmmsg-reuse"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        for chunk in self.gso_payload.chunks_exact_mut(payload.len()) {
            chunk.copy_from_slice(payload);
        }

        let fd = self.socket.as_raw_fd();
        let mut remaining = packets;

        while remaining > 0 {
            let datagrams_this_batch = remaining.min(self.datagrams_per_batch as u64) as usize;
            let mut datagrams_left_in_batch = datagrams_this_batch;
            let mut messages_this_batch = 0;

            while datagrams_left_in_batch > 0 {
                let segments = datagrams_left_in_batch.min(self.segments_per_message);
                self.iovecs[messages_this_batch].iov_len = segments * payload.len();
                datagrams_left_in_batch -= segments;
                messages_this_batch += 1;
            }

            let sent = unsafe {
                libc::sendmmsg(
                    fd,
                    self.messages.as_mut_ptr(),
                    messages_this_batch as libc::c_uint,
                    0,
                )
            };

            if sent < 0 {
                bail!("sendmmsg failed: {}", std::io::Error::last_os_error());
            }

            if sent == 0 {
                bail!("sendmmsg sent zero UDP GSO messages");
            }

            let sent = sent as usize;
            let full_messages = sent.saturating_sub(1);
            let mut sent_datagrams = full_messages * self.segments_per_message;

            let last_len = self.messages[sent - 1].msg_len as usize;
            sent_datagrams += last_len / payload.len();

            if sent_datagrams == 0 {
                bail!("sendmmsg reported no UDP GSO datagrams sent");
            }

            remaining -= sent_datagrams as u64;
        }

        Ok(())
    }
}
