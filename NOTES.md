std send_to
std connected
std connected (pinned to fast core)
libc send_to
libc send
sendmmsg
sendmmsg (memory optimizations with reusable buffers)
sendmmsg (multithreaded pinned to fast cores)
io_uring
io_uring (zero-copy)
kernel bypass

sudo systemctl disable --now irqbalance
sudo cpupower frequency-set -g performance
sudo x86_energy_perf_policy performance
GRUB_CMDLINE_LINUX_DEFAULT
    nosmt
    isolcpus=domain,managed_irq,4,6
    nohz_full=4,6
    rcu_nocbs=4,6
    irqaffinity=0-3,8-19
    intel_idle.max_cstate=1
    processor.max_cstate=1


sudo update-grub
sudo reboot