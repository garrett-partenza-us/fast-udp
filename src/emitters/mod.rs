pub mod std_send_to;
pub mod std_connected;
pub mod std_connected_pin;

use anyhow::Result;

pub trait UdpEmitter {
    fn name(&self) -> &'static str;

    fn send_many(&mut self, parload: &[u8], packets: u64) -> Result<()>;
}
