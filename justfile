set shell := ["bash", "-cu"]

packets := "1000000"
payload_size := "1200"
target := "127.0.0.1:9000"
cpu := "6"

build:
    cargo build --release

bench-std-send-to: build
    ./target/release/fast-udp sink --bind {{target}} & \
    sink_pid=$!; \
    trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; \
    sleep 0.2; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-std-send-to.md \
      --export-json target/bench-std-send-to.json \
      './target/release/fast-udp emit --implementation std-send-to --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-std-connected: build
    ./target/release/fast-udp sink --bind {{target}} & \
    sink_pid=$!; \
    trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; \
    sleep 0.2; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-std-connected.md \
      --export-json target/bench-std-connected.json \
      './target/release/fast-udp emit --implementation std-connected --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'

bench-std-connected-pin: build
    ./target/release/fast-udp sink --bind {{target}} & \
    sink_pid=$!; \
    trap 'kill "$sink_pid" 2>/dev/null || true' EXIT; \
    sleep 0.2; \
    hyperfine \
      --warmup 3 \
      --runs 20 \
      --export-markdown target/bench-std-connected-pin.md \
      --export-json target/bench-std-connected-pin.json \
      './target/release/fast-udp emit --implementation std-connected-pin --cpu {{cpu}} --packets {{packets}} --payload-size {{payload_size}} --target {{target}}'
