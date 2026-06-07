# fast-udp

UDP emitter benchmarks in Rust.

The goal is to compare increasingly lower-level UDP send paths under the same
workload:

- 1,000,000 logical UDP packets
- 1200 byte payload
- emitter pinned to CPU 6
- local sink pinned to CPU 4
- loopback target: `127.0.0.1:9000`
- `hyperfine --warmup 3 --runs 20`

This is primarily a local Linux UDP stack benchmark. Remote runs are supported,
but those measure the network path as much as the implementation.

## Implementations

| Implementation | What it does |
| --- | --- |
| `std-send-to` | `std::net::UdpSocket::send_to` per packet. Destination passed every call. |
| `std-connected` | Connected UDP socket, then `UdpSocket::send` per packet. |
| `libc-send` | Connected UDP socket, then direct `libc::send` per packet. |
| `libc-sendmmsg` | Uses `sendmmsg` to submit batches of UDP datagrams. |
| `libc-sendmmsg-reuse` | Same as `sendmmsg`, but reuses `mmsghdr` and `iovec` storage. |
| `libc-udp-gso` | Uses `UDP_SEGMENT` so one send carries multiple logical UDP packets. |
| `libc-udp-gso-sendmmsg-reuse` | Combines UDP GSO with reused `sendmmsg` message storage. |
| `io-uring` | Uses io_uring with UDP GSO; one SQE sends one GSO aggregate buffer. |

Linux-only implementations are behind `cfg(target_os = "linux")`. macOS can
build and run the sink and the std emitters.

## File layout

```text
src/main.rs                         CLI and implementation selection
src/sink.rs                         UDP sink with live packet/byte counters
src/emitters/mod.rs                 emitter trait and CPU affinity helper
src/emitters/std_send_to.rs         std send_to baseline
src/emitters/std_connected.rs       std connected baseline
src/emitters/libc_send.rs           direct libc send
src/emitters/libc_sendmmsg.rs       sendmmsg batching
src/emitters/libc_sendmmsg_reuse.rs sendmmsg with reused message storage
src/emitters/libc_udp_gso.rs        UDP GSO
src/emitters/libc_udp_gso_sendmmsg_reuse.rs
src/emitters/io_uring.rs            io_uring + UDP GSO
justfile                            build, sink, and benchmark recipes
NOTES.md                            host setup notes
```

## Running

Install:

```bash
cargo install just
cargo install hyperfine
```

Build:

```bash
just build
```

Run a local benchmark with an auto-started local sink:

```bash
just bench-std-connected
just bench-libc-udp-gso
just bench-io-uring
```

Run a visible sink:

```bash
just sink
```

Remote sink:

```bash
./target/release/fast-udp sink --bind 0.0.0.0:9000
```

Remote benchmark from the sender:

```bash
just local_sink=false target=192.168.1.64:9000 bench-io-uring
```

Override common parameters:

```bash
just packets=500000 payload_size=1200 batch_size=128 cpus=6 sink_cpu=4 bench-io-uring
```

## Results

Measured on the host described in `NOTES.md`.

Payload throughput is calculated from payload bytes only:

```text
1,000,000 packets * 1200 bytes
```

| Implementation | Mean | Stddev | Packets/s | Payload Gbit/s |
| --- | ---: | ---: | ---: | ---: |
| `std-send-to` | 1.775 s | 0.011 s | 563,292 | 5.41 |
| `std-connected` | 1.644 s | 0.009 s | 608,177 | 5.84 |
| `libc-send` | 1.629 s | 0.008 s | 613,737 | 5.89 |
| `libc-sendmmsg` | 1.441 s | 0.019 s | 693,751 | 6.66 |
| `libc-sendmmsg-reuse` | 1.453 s | 0.027 s | 688,265 | 6.61 |
| `libc-udp-gso` | 233.6 ms | 1.2 ms | 4,281,542 | 41.10 |
| `libc-udp-gso-sendmmsg-reuse` | 241.5 ms | 0.7 ms | 4,140,373 | 39.75 |
| `io-uring` | 231.1 ms | 1.5 ms | 4,326,295 | 41.53 |

The large step is UDP GSO. It reduces logical per-packet work by sending one
larger buffer and asking the kernel to segment it into UDP datagrams.

`io-uring` is only competitive here after using UDP GSO. Plain one-packet-per-SQE
io_uring is not the interesting path for this workload.
