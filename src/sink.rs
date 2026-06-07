use std::{
    io::{self, Write},
    net::UdpSocket,
    time::{Duration, Instant},
};

use anyhow::Result;

pub fn run(bind: &str) -> Result<()> {
    let socket = UdpSocket::bind(bind)?;
    let mut buffer = [0_u8; 65_507];
    let mut packets = 0_u64;
    let mut bytes = 0_u64;
    let started = Instant::now();
    let mut last_print = Instant::now();

    loop {
        let (received, _) = socket.recv_from(&mut buffer)?;
        packets += 1;
        bytes += received as u64;

        if last_print.elapsed() >= Duration::from_millis(250) {
            let elapsed = started.elapsed().as_secs_f64();
            let packets_per_second = packets as f64 / elapsed;
            let bytes_per_second = bytes as f64 / elapsed;

            print!(
                "\rpackets={} bytes={} pps={:.0} mbps={:.2}",
                packets,
                bytes,
                packets_per_second,
                bytes_per_second * 8.0 / 1_000_000.0
            );
            io::stdout().flush()?;
            last_print = Instant::now();
        }
    }
}
