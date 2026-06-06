use std::net::UdpSocket;

use anyhow::Result;

pub fn run(bind: &str) -> Result<()> {
    let socket = UdpSocket::bind(bind)?;
    let mut buffer = [0_u8; 65_507];

    loop {
        socket.recv_from(&mut buffer)?;
    }
}