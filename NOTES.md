# Host setup

Benchmark host:

```text
Intel NUC
CPU: 13th Gen Intel Core i9-13900H
Kernel: Linux 6.8.0-111-generic
Rust: rustc 1.95.0
hyperfine: 1.20.0
just: 1.49.0
```

CPU topology after disabling SMT:

```text
CPU CORE ONLINE MAXMHZ
0   0    yes    5200
2   1    yes    5200
4   2    yes    5400
6   3    yes    5400
8   4    yes    5200
10  5    yes    5200
12  6    yes    4100
13  7    yes    4100
14  8    yes    4100
15  9    yes    4100
16  10   yes    4100
17  11   yes    4100
18  12   yes    4100
19  13   yes    4100
```

Benchmark placement:

```text
CPU 6: emitter
CPU 4: local sink
```

## Kernel command line

Current command line:

```text
quiet splash nosmt isolcpus=domain,managed_irq,4,6 nohz_full=4,6 rcu_nocbs=4,6 irqaffinity=0-3,8-19 intel_idle.max_cstate=1 processor.max_cstate=1
```

Full observed `/proc/cmdline`:

```text
BOOT_IMAGE=/boot/vmlinuz-6.8.0-111-generic root=UUID=df7c9add-f950-4464-8aae-f7640bda7171 ro quiet splash nosmt isolcpus=domain,managed_irq,4,6 nohz_full=4,6 rcu_nocbs=4,6 irqaffinity=0-3,8-19 intel_idle.max_cstate=1 processor.max_cstate=1 vt.handoff=7
```

Meaning:

```text
nosmt                         disable hyperthreading
isolcpus=domain,managed_irq   keep normal scheduler work off CPUs 4,6
nohz_full=4,6                 reduce scheduler ticks on CPUs 4,6
rcu_nocbs=4,6                 move RCU callbacks off CPUs 4,6
irqaffinity=0-3,8-19          keep default IRQs off CPUs 4,6
intel_idle.max_cstate=1       limit Intel idle states to C1
processor.max_cstate=1        generic ACPI C-state fallback
```

## Runtime tuning

Run after boot:

```bash
sudo systemctl disable --now irqbalance
sudo cpupower frequency-set -g performance
sudo x86_energy_perf_policy performance
```

Observed:

```text
irqbalance: inactive / disabled
SMT: off
isolated CPUs: 4,6
nohz_full CPUs: 4,6
governor: performance
```

Verify:

```bash
cat /sys/devices/system/cpu/smt/control
cat /sys/devices/system/cpu/isolated
cat /sys/devices/system/cpu/nohz_full
systemctl is-active irqbalance
systemctl is-enabled irqbalance
cpupower frequency-info | grep -E 'governor|boost|current CPU frequency'
```

Watch benchmark cores:

```bash
mpstat -P 4,6 1
watch -n 0.2 'ps -eLo pid,tid,psr,pcpu,comm | awk "$3 == 4 || $3 == 6 {print}"'
```

## Network/offload notes

Loopback supports UDP GSO:

```text
lo:
generic-segmentation-offload: on
tx-udp-segmentation: on
tx-gso-list: on
```

Physical interface:

```text
interface: enp1s0
speed: 2500Mb/s
duplex: full
```

The physical NIC reported plain `tx-udp-segmentation: off [fixed]` during testing.
Loopback GSO results should not be treated as physical NIC GSO results.

## Remote receiver

macOS can build the sink because Linux-only emitter modules are behind
`cfg(target_os = "linux")`.

Receiver:

```bash
cargo build --release
./target/release/fast-udp sink --bind 0.0.0.0:9000
```

Sender:

```bash
just local_sink=false target=<receiver-ip>:9000 bench-io-uring
```

Remote tests are network-path tests. On a 2.5Gb/s sender link, sending
1,000,000 packets with 1200 byte payload is 1.2 GB payload, or 9.6 Gbit before
headers. The theoretical lower bound at 2.5Gb/s is about 3.84 seconds.
