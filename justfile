set shell := ["bash", "-cu"]

packets := "1000000"
payload_size := "1200"
target := "127.0.0.1:9000"
cpus := "6"
sink_cpu := "4"
batch_size := "128"
local_sink := "true"

build:
    cargo build --release

sink: build
    ./target/release/fast-udp sink --bind 0.0.0.0:9000

bench-std-send-to: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-std-send-to.md \
      --export-json target/bench-std-send-to.json \
      './target/release/fast-udp emit --implementation std-send-to --cpus {{cpus}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-std-connected: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-std-connected.md \
      --export-json target/bench-std-connected.json \
      './target/release/fast-udp emit --implementation std-connected --cpus {{cpus}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-libc-send: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-libc-send.md \
      --export-json target/bench-libc-send.json \
      './target/release/fast-udp emit --implementation libc-send --cpus {{cpus}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-libc-sendmmsg: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-libc-sendmmsg.md \
      --export-json target/bench-libc-sendmmsg.json \
      './target/release/fast-udp emit --implementation libc-sendmmsg --cpus {{cpus}} --batch-size {{batch_size}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-libc-sendmmsg-reuse: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-libc-sendmmsg-reuse.md \
      --export-json target/bench-libc-sendmmsg-reuse.json \
      './target/release/fast-udp emit --implementation libc-sendmmsg-reuse --cpus {{cpus}} --batch-size {{batch_size}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-libc-udp-gso: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-libc-udp-gso.md \
      --export-json target/bench-libc-udp-gso.json \
      './target/release/fast-udp emit --implementation libc-udp-gso --cpus {{cpus}} --batch-size {{batch_size}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-libc-udp-gso-sendmmsg-reuse: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-libc-udp-gso-sendmmsg-reuse.md \
      --export-json target/bench-libc-udp-gso-sendmmsg-reuse.json \
      './target/release/fast-udp emit --implementation libc-udp-gso-sendmmsg-reuse --cpus {{cpus}} --batch-size {{batch_size}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-io-uring: build
    if [[ "{{local_sink}}" == "true" ]]; then taskset -c {{sink_cpu}} ./target/release/fast-udp sink --bind {{target}} >/dev/null 2>&1 & sink_pid=$!; trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; sleep 0.2; fi; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-io-uring.md \
      --export-json target/bench-io-uring.json \
      './target/release/fast-udp emit --implementation io-uring --cpus {{cpus}} --batch-size {{batch_size}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-std: bench-std-connected
