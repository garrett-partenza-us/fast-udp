use std::net::SocketAddr;

use anyhow::Result;
use clap::Parser;
use fast_udp::emitters::{
    UdpEmitter, pin_current_thread_to_cpus, std_connected::StdConnectedEmitter,
    std_send_to::StdSendToEmitter,
};

#[cfg(target_os = "linux")]
use fast_udp::emitters::{
    io_uring::IoUringEmitter, libc_send::LibcSendEmitter, libc_sendmmsg::LibcSendmmsgEmitter,
    libc_sendmmsg_reuse::LibcSendmmsgReuseEmitter, libc_udp_gso::LibcUdpGsoEmitter,
    libc_udp_gso_sendmmsg_reuse::LibcUdpGsoSendmmsgReuseEmitter,
};

#[derive(clap::Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    Emit {
        #[arg(long, value_enum, default_value_t = Implementation::StdConnected)]
        implementation: Implementation,

        #[arg(long, default_value_t = 1_000_000)]
        packets: u64,

        #[arg(long, default_value_t = 1200)]
        payload_size: usize,

        #[arg(long, default_value = "127.0.0.1:9000")]
        target: String,

        #[arg(long, value_delimiter = ',')]
        cpus: Vec<usize>,

        #[arg(long, default_value_t = 128)]
        batch_size: usize,
    },
    Sink {
        #[arg(long, default_value = "127.0.0.1:9000")]
        bind: String,
    },
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Implementation {
    StdSendTo,
    StdConnected,
    #[cfg(target_os = "linux")]
    LibcSend,
    #[cfg(target_os = "linux")]
    LibcSendmmsg,
    #[cfg(target_os = "linux")]
    LibcSendmmsgReuse,
    #[cfg(target_os = "linux")]
    LibcUdpGso,
    #[cfg(target_os = "linux")]
    LibcUdpGsoSendmmsgReuse,
    #[cfg(target_os = "linux")]
    IoUring,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Emit {
            implementation,
            packets,
            payload_size,
            target,
            cpus,
            batch_size,
        } => {
            pin_current_thread_to_cpus(&cpus)?;

            let target: SocketAddr = target.parse()?;
            let payload = vec![0_u8; payload_size];

            let mut emitter: Box<dyn UdpEmitter> = match implementation {
                Implementation::StdSendTo => Box::new(StdSendToEmitter::new(target)?),
                Implementation::StdConnected => Box::new(StdConnectedEmitter::new(target)?),
                #[cfg(target_os = "linux")]
                Implementation::LibcSend => Box::new(LibcSendEmitter::new(target)?),
                #[cfg(target_os = "linux")]
                Implementation::LibcSendmmsg => {
                    Box::new(LibcSendmmsgEmitter::new(target, batch_size)?)
                }
                #[cfg(target_os = "linux")]
                Implementation::LibcSendmmsgReuse => {
                    Box::new(LibcSendmmsgReuseEmitter::new(target, batch_size)?)
                }
                #[cfg(target_os = "linux")]
                Implementation::LibcUdpGso => {
                    Box::new(LibcUdpGsoEmitter::new(target, batch_size, payload_size)?)
                }
                #[cfg(target_os = "linux")]
                Implementation::LibcUdpGsoSendmmsgReuse => Box::new(
                    LibcUdpGsoSendmmsgReuseEmitter::new(target, batch_size, payload_size)?,
                ),
                #[cfg(target_os = "linux")]
                Implementation::IoUring => {
                    Box::new(IoUringEmitter::new(target, batch_size, payload_size)?)
                }
            };

            emitter.send_many(&payload, packets)?;
        }
        Command::Sink { bind } => {
            fast_udp::sink::run(&bind)?;
        }
    }

    Ok(())
}
