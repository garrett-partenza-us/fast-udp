use std::{
    net::{SocketAddr, UdpSocket},
    os::fd::AsRawFd,
};

use anyhow::{Result, bail};

use crate::emitters::UdpEmitter;

pub struct LibcSendmmsgEmitter {
    socket: UdpSocket,
    batch_size: usize,
}

impl LibcSendmmsgEmitter {
    pub fn new(target: SocketAddr, batch_size: usize) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(target)?;

        Ok(Self { socket, batch_size })
    }
}

impl UdpEmitter for LibcSendmmsgEmitter {
    fn name(&self) -> &'static str {
        "libc-sendmmsg"
    }

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()> {
        let fd = self.socket.as_raw_fd();
        let batch_size = self.batch_size.min(packets as usize);

        let mut iovecs = vec![
            libc::iovec {
                iov_base: payload.as_ptr() as *mut libc::c_void,
                iov_len: payload.len(),
            };
            batch_size
        ];

        let mut messages = vec![
            libc::mmsghdr {
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
            };
            batch_size
        ];

        for index in 0..batch_size {
            messages[index].msg_hdr.msg_iov = &mut iovecs[index];
        }

        let mut remaining = packets;

        while remaining > 0 {
            let this_batch = remaining.min(batch_size as u64) as usize;

            let sent =
                unsafe { libc::sendmmsg(fd, messages.as_mut_ptr(), this_batch as libc::c_uint, 0) };

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
