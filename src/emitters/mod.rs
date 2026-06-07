#[cfg(target_os = "linux")]
pub mod io_uring;
#[cfg(target_os = "linux")]
pub mod libc_send;
#[cfg(target_os = "linux")]
pub mod libc_sendmmsg;
#[cfg(target_os = "linux")]
pub mod libc_sendmmsg_reuse;
#[cfg(target_os = "linux")]
pub mod libc_udp_gso;
#[cfg(target_os = "linux")]
pub mod libc_udp_gso_sendmmsg_reuse;
pub mod std_connected;
pub mod std_send_to;

use anyhow::{Result, bail};

pub trait UdpEmitter {
    fn name(&self) -> &'static str;

    fn send_many(&mut self, payload: &[u8], packets: u64) -> Result<()>;
}

#[cfg(target_os = "linux")]
pub fn pin_current_thread_to_cpus(cpus: &[usize]) -> Result<()> {
    if cpus.is_empty() {
        return Ok(());
    }

    unsafe {
        let mut set: libc::cpu_set_t = std::mem::zeroed();

        libc::CPU_ZERO(&mut set);

        for &cpu in cpus {
            libc::CPU_SET(cpu, &mut set);
        }

        let result = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);

        if result != 0 {
            bail!(
                "sched_setaffinity failed: {}",
                std::io::Error::last_os_error()
            );
        }
    }

    Ok(())
}

#[cfg(not(target_os = "linux"))]
pub fn pin_current_thread_to_cpus(cpus: &[usize]) -> Result<()> {
    if cpus.is_empty() {
        Ok(())
    } else {
        bail!("CPU affinity is only supported on Linux")
    }
}
