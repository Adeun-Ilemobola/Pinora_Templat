# Pinora Desktop UI

The Pinora desktop application is a native Rust application built with
[Slint](https://slint.dev/). It connects to Pinora hardware over serial,
decodes newline-delimited protocol messages, maintains module state, and
publishes supported updates to the UI event loop.

## Run

Install Rust, then run from this directory:

```bash
cargo run --locked
```

The connection form enumerates serial ports at startup. Select a port and baud
rate, then connect. Wi-Fi and Bluetooth forms are present but their transports
are currently placeholders.

## Validate

```bash
cargo check --locked
cargo clippy --locked
```

Slint sources live under `ui/`; Rust transport, controller, and UI publication
code lives under `src/`.
