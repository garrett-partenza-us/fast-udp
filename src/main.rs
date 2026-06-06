use std::net::{SocketAddr};

use anyhow::Result;
use clap::Parser;
use fast_udp::emitters::{
    std_connected::StdConnectedEmitter,
    std_connected_pin::StdConnectedPinEmitter,
    std_send_to::StdSendToEmitter,
    UdpEmitter,
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

        #[arg(long, default_value_t = 0)]
        cpu: usize,
    },
    Sink {
        #[arg(long, default_value = "127.0.0.1:9000")]
        bind: String,
    }
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum Implementation {
    StdSendTo,
    StdConnected,
    StdConnectedPin,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Emit {
            implementation,
            packets,
            payload_size,
            target,
            cpu
        } => {
            let target: SocketAddr = target.parse()?;
            let payload = vec![0_u8; payload_size];

            let mut emitter: Box<dyn UdpEmitter> = match implementation {
                Implementation::StdSendTo => Box::new(StdSendToEmitter::new(target)?),
                Implementation::StdConnected => Box::new(StdConnectedEmitter::new(target)?),
                Implementation::StdConnectedPin => {
                    Box::new(StdConnectedPinEmitter::new(target, cpu)?)
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
