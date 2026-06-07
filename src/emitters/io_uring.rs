use std::{
    net::{SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use anyhow::{Result, bail};
use io_uring::{IoUring, opcode, types};

use crate::emitters::UdpEmitter;

const UDP_SEGMENT: libc::c_int = 103;
const MAX_GSO_SEGMENTS: usize = 64;
const MAX_UDP_PAYLOAD_BYTES: usize = 65_507;

pub struct IoUringEmitter {
    socket: UdpSocket,
    ring: IoUring,
    queue_depth: usize,
    segments_per_send: usize,
    gso_payload: Vec<u8>,
}

impl IoUringEmitter {
    pub fn new(target: SocketAddr, queue_depth: usize, segment_size: usize) -> Result<Self> {
        if segment_size == 0 {
            bail!("UDP GSO segment size must be greater than zero");
        }

        let queue_depth = queue_depth.max(1);
        let segments_per_send = queue_depth
            .min(MAX_GSO_SEGMENTS)
            .min(MAX_UDP_PAYLOAD_BYTES / segment_size);

        if segments_per_send == 0 {
            bail!("UDP GSO segment size is too large: {segment_size}");
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
            ring: IoUring::new(queue_depth as u32)?,
            queue_depth,
            segments_per_send,
            gso_payload: vec![0_u8; segment_size * segments_per_send],
        })
    }
}

impl UdpEmitter for IoUringEmitter {
    fn name(&self) -> &'static str {
        "io-uring"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        for chunk in self.gso_payload.chunks_exact_mut(payload.len()) {
            chunk.copy_from_slice(payload);
        }

        let fd = self.socket.as_raw_fd();
        let mut submitted_datagrams = 0_u64;
        let mut completed_datagrams = 0_u64;

        while completed_datagrams < packets {
            let mut submitted_messages = 0_usize;

            {
                let mut submission_queue = self.ring.submission();

                while submitted_datagrams < packets && submitted_messages < self.queue_depth {
                    let segments =
                        (packets - submitted_datagrams).min(self.segments_per_send as u64) as usize;
                    let bytes_to_send = segments * payload.len();

                    let entry = opcode::Send::new(
                        types::Fd(fd),
                        self.gso_payload.as_ptr(),
                        bytes_to_send as _,
                    )
                    .build()
                    .user_data(bytes_to_send as u64);

                    unsafe {
                        submission_queue
                            .push(&entry)
                            .map_err(|_| anyhow::anyhow!("io_uring submission queue is full"))?;
                    }

                    submitted_datagrams += segments as u64;
                    submitted_messages += 1;
                }
            }

            self.ring.submit_and_wait(submitted_messages)?;

            let mut completed_messages = 0_usize;

            while completed_messages < submitted_messages {
                {
                    let completion_queue = self.ring.completion();

                    for completion in completion_queue {
                        let result = completion.result();
                        let expected_bytes = completion.user_data() as usize;

                        if result < 0 {
                            bail!(
                                "io_uring UDP GSO send failed: {}",
                                std::io::Error::from_raw_os_error(-result)
                            );
                        }

                        if result as usize != expected_bytes {
                            bail!(
                                "partial io_uring UDP GSO send: sent {} bytes out of {}",
                                result,
                                expected_bytes
                            );
                        }

                        completed_datagrams += (expected_bytes / payload.len()) as u64;
                        completed_messages += 1;
                    }
                }

                if completed_messages < submitted_messages {
                    self.ring.submit_and_wait(1)?;
                }
            }
        }

        Ok(())
    }
}
