use std::{
    net::{SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use anyhow::{Result, bail};

use crate::emitters::UdpEmitter;

pub struct LibcSendmmsgReuseEmitter {
    socket: UdpSocket,
    iovecs: Vec<libc::iovec>,
    messages: Vec<libc::mmsghdr>,
}

impl LibcSendmmsgReuseEmitter {
    pub fn new(target: SocketAddr, batch_size: usize) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        let mut iovecs = Vec::with_capacity(batch_size);
        let mut messages = Vec::with_capacity(batch_size);

        for _ in 0..batch_size {
            iovecs.push(libc::iovec {
                iov_base: std::ptr::null_mut(),
                iov_len: 0,
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

        for index in 0..batch_size {
            messages[index].msg_hdr.msg_iov = &mut iovecs[index];
        }

        Ok(Self {
            socket,
            iovecs,
            messages,
        })
    }
}

impl UdpEmitter for LibcSendmmsgReuseEmitter {
    fn name(&self) -> &'static str {
        "libc-sendmmsg-reuse"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        let fd = self.socket.as_raw_fd();
        let batch_size = self.messages.len();

        for iovec in &mut self.iovecs {
            iovec.iov_base = payload.as_ptr() as *mut libc::c_void;
            iovec.iov_len = payload.len();
        }

        let mut remaining = packets;

        while remaining > 0 {
            let this_batch = remaining.min(batch_size as u64) as usize;

            let sent = unsafe {
                libc::sendmmsg(
                    fd,
                    self.messages.as_mut_ptr(),
                    this_batch as libc::c_uint,
                    0,
                )
            };

            if sent < 0 {
                bail!("sendmmsg failed: {}", std::io::Error::last_os_error());
            }

            if sent == 0 {
                bail!("sendmmsg sent zero datagrams");
            }

            remaining -= sent as u64;
        }

        Ok(())
    }
}
