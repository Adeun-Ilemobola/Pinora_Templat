<div align="center">

# Pinora

### A Slint desktop client, shared Rust protocol, and modular ESP32 firmware

Connect. Inspect. Control.

![Version](https://img.shields.io/badge/version-0.1.0-7c3aed?style=for-the-badge)
![Status](https://img.shields.io/badge/status-PRE--ALPHA-f97316?style=for-the-badge)
[![Slint](https://img.shields.io/badge/Slint-1.16.1-2379F4?style=for-the-badge)](https://slint.dev/)
[![Rust](https://img.shields.io/badge/language-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![ESP32](https://img.shields.io/badge/target-ESP32-E7352C?style=for-the-badge&logo=espressif&logoColor=white)](https://www.espressif.com/en/products/socs/esp32)

> [!WARNING]
> **Pinora is a pre-alpha prototype.** The UI, transport layer, wire protocol,
> firmware modules, and hardware configuration are still evolving. Do not use
> the project for production or safety-critical systems.

</div>

## What is Pinora?

Pinora is an experimental Rust system for connecting a desktop application to
modular hardware running on an ESP32. After the latest refactor, the repository
is split into three Rust crates:

- `UI/` — a native desktop application built with Slint
- `protocol/` — shared Serde message and module definitions
- `Firmware_Templates/` — ESP-IDF firmware and hardware modules

The shared `pinora-protocol` crate replaces the previous duplicated Rust and
TypeScript protocol definitions. Both the desktop and firmware crates depend on
it through a local path dependency.

The project is currently focused on rebuilding the connection and protocol
foundation. The Slint application provides transport-specific connection forms
and a working serial connection path. The firmware reads newline-delimited JSON
commands from its console UART and emits protocol messages as JSON lines.

## Project status

| Area | Current state |
|---|---|
| Project version | `0.1.0` |
| Maturity | Pre-alpha |
| Desktop stack | Rust 2024, Slint `1.16.1`, `serialport` `4.10` |
| Firmware stack | Rust 2021, ESP-IDF `v5.5.3`, `esp-idf-svc` `0.52.1` |
| Firmware target | ESP32 / `xtensa-esp32-espidf` |
| Firmware toolchain | `esp-1.93` |
| Shared protocol | `pinora-protocol` `0.1.0`, Rust 2024, Serde JSON |
| Working desktop transport | Serial |
| Firmware wire format | One JSON object per line over the console UART |
| Active firmware module | Infrared remote receiver on GPIO 12 |

## Architecture

```mermaid
flowchart LR
    SLINT["Slint views and components"] <-->|"properties and callbacks"| BRIDGE["Rust UI bridge"]
    BRIDGE <--> GATE["Transport gateway"]
    GATE --> SERIAL["Serial transport"]
    GATE -.-> WIFI["Wi-Fi stub"]
    GATE -.-> BT["Bluetooth stub"]
    SERIAL <-->|"newline-delimited bytes / JSON"| UART["ESP32 console UART"]

    PROTOCOL["pinora-protocol"] --> UI["Desktop crate"]
    PROTOCOL --> FW["Firmware crate"]
    UART <--> FW
    FW --> REGISTRY["Runtime module registry"]
    FW --> EMITTER["Bounded event emitter"]
    REGISTRY --> MODULES["Hardware modules"]
```

### Desktop application

The desktop application is now a native Rust binary rather than an
Electrobun/React application.

- `build.rs` compiles `ui/app-window.slint` with `slint-build`.
- `src/main.rs` creates the Slint window and binds its callbacks to Rust.
- `src/ui_bridge/transport_form.rs` maps form values and connection actions
  between Slint and the transport layer.
- `src/transport/transport_gate.rs` owns the selected transport behind a single
  gateway.
- `src/transport/serial_transport.rs` enumerates ports, opens the selected port,
  and reads newline-terminated data on a background thread.

The current window lets the user choose Serial, Wi-Fi, or Bluetooth and displays
the associated form. Serial connections are implemented. The Wi-Fi and
Bluetooth types are placeholders, and their form submissions do not yet create
working connections.

Incoming serial lines currently go to diagnostic output. Protocol decoding,
module state, command writing, and module-specific Slint views are still to be
connected.

### Shared protocol

`protocol/` is the common wire-contract crate. It owns:

- `IncomingCommand` and the currently enabled command variants
- `ProtocolMessage`, registrations, module events, and system information
- module types and module-specific payloads
- Serde serialization and deserialization for JSON transport

The command enum currently enables LED, stepper motor, and RFID commands. The
event enum currently enables LED, button, system log, stepper motor, IMU, and
RFID events. LiDAR, servo, and rangefinder payload types exist, but their top-level
command/event variants are currently disabled while those paths are refactored.

### Firmware

The firmware uses ESP-IDF through `esp-idf-svc` and is organized around a small
module runtime:

- `ModuleCore` assigns runtime UUIDs and stores each module's type and readable ID.
- The `Module` trait provides ticking, command handling, registration, and event
  emission hooks.
- A runtime `HashMap` dispatches incoming commands to modules by UUID.
- `Emitter` uses a bounded channel and a background thread to serialize outgoing
  `ProtocolMessage` values.
- A separate reader thread parses one `IncomingCommand` from each UART line.

The active `main.rs` configuration instantiates only the infrared remote receiver
module. It samples GPIO 12 and decodes the timings of a 32-bit remote frame. Other
module implementations are present but are either not instantiated or currently
disabled from the firmware module tree.

## Repository layout

```text
.
├── UI/                              # Native Slint desktop crate
│   ├── .cargo/config.toml           # Larger Windows debug stack
│   ├── src/
│   │   ├── main.rs                  # Application entry point
│   │   ├── ui_bridge/               # Slint callback/property bindings
│   │   └── transport/               # Serial and placeholder transports
│   ├── ui/
│   │   ├── app-window.slint         # Main window and connection form
│   │   └── component/               # Reusable Slint components
│   ├── build.rs                     # Slint compile step
│   └── Cargo.toml
├── protocol/                        # Shared pinora-protocol crate
│   ├── src/
│   │   ├── command.rs               # Incoming command envelope
│   │   ├── registration.rs          # Outgoing protocol envelope
│   │   ├── module_event.rs          # Event variants and system logs
│   │   ├── global_definitions.rs    # Shared module identifiers
│   │   └── module/                  # Module-specific wire types
│   └── Cargo.toml
├── Firmware_Templates/              # ESP32 firmware crate
│   ├── .cargo/config.toml           # ESP target, linker, runner, and IDF version
│   ├── src/
│   │   ├── core/                    # Emitter, module core, hardware helpers
│   │   ├── module/                  # Hardware module implementations
│   │   ├── utilities/               # Shared firmware utilities
│   │   └── main.rs                  # Hardware setup and runtime loop
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── sdkconfig.defaults
├── pinora.toml                      # Pinora project metadata (UI path is stale)
├── justfile                         # Helper recipes (UI recipes need migration)
└── README.md
```

> [!NOTE]
> The repository uses a multi-crate layout with path dependencies, but it does
> not currently contain a root Cargo workspace manifest. Run Cargo commands from
> the individual crate directories as shown below.
>
> The UI path in `pinora.toml` and the `frontend`/`buildUI` recipes in the
> `justfile` still reference the removed `UI_Templates/` directory. Until
> those helpers are migrated, use the direct Cargo commands in this README.

## Getting started

### Prerequisites

For the shared protocol and desktop application:

- A recent stable Rust toolchain with Rust 2024 edition support
- Native build tools required by Rust on your operating system
- Permission to access the target serial port
- On Linux, the development packages required by the `serialport` crate, commonly
  `pkg-config` and `libudev-dev`

For the firmware, install the
[ESP Rust development environment](https://docs.esp-rs.org/book/installation/index.html)
and make sure `ldproxy` and `espflash` are available. The firmware directory pins
the `esp-1.93` toolchain and configures the `xtensa-esp32-espidf` target.

### Check the shared protocol

```bash
cd protocol
cargo check
```

### Run the Slint desktop application

```bash
cd UI
cargo run
```

To create an optimized desktop build:

```bash
cd UI
cargo build --release
```

The application enumerates serial ports when it starts. Select a transport,
choose the required values, and use the connection button. The serial baud-rate
selector supports `9600`, `19200`, `38400`, `57600`, `115200`, `230400`,
`460800`, and `921600`; an invalid or empty value falls back to `115200`.

### Build and flash the firmware

```bash
cd Firmware_Templates
cargo build
cargo run --release
```

The configured Cargo runner turns `cargo run --release` into an `espflash flash
--monitor` operation. Stop the serial monitor before connecting from the desktop
application because only one process can normally own the serial port at a time.

To build without flashing:

```bash
cd Firmware_Templates
cargo build --release
```

## Current hardware configuration

The active firmware entry point currently configures:

- ESP32 console UART 0 with 2048-byte RX and TX buffers
- An infrared remote receiver input on GPIO 12
- SPI2 at 1 MHz with SCK on GPIO 18, MOSI on GPIO 19, MISO on GPIO 23, and CS
  on GPIO 5

The SPI driver is initialized for the in-progress RFID path, but the RFID module
itself is not currently instantiated. The earlier LiDAR/I²C configuration is
still present in source as disabled code and is not part of the active firmware
runtime.

## Wire protocol

Pinora uses JSON messages separated by newlines:

- Commands travel from the desktop side to the firmware as `IncomingCommand`.
- Registrations, module events, and system information travel from the firmware
  through `ProtocolMessage`.
- Runtime modules are addressed by generated UUID.

The current protocol source of truth is `protocol/src/`.

### Command envelope

```json
{"id":"module-runtime-uuid","module_type":"Led","payload":{"command":"Toggle"}}
```

### Registration envelope

```json
{
  "type": "Registration",
  "payload": {
    "id": "generated-runtime-uuid",
    "module_type": "Led",
    "lool_up_id": "status_led",
    "parent_id": ""
  }
}
```

> [!NOTE]
> `lool_up_id` is the current serialized field name. Its spelling is retained for
> wire compatibility during pre-alpha development.

### Module event envelope

```json
{
  "type": "ModuleEvent",
  "payload": {
    "module_type": "Led",
    "event": {
      "event_type": "Brightness",
      "id": "module-runtime-uuid",
      "level": 80
    }
  }
}
```

## Implementation matrix

| Area | Protocol types | Firmware implementation | Active at startup | Slint UI |
|---|:---:|:---:|:---:|:---:|
| Serial transport | N/A | Console UART | Yes | Working connection/read path |
| Wi-Fi transport | N/A | Emitter placeholder | No | Form and stub only |
| Bluetooth transport | N/A | Emitter placeholder | No | Form and stub only |
| Infrared remote receiver | Module identifier only | Yes | Yes | Not yet exposed |
| LED | Commands and events | Yes | No | Not yet exposed |
| Button | Events | Yes | No | Not yet exposed |
| Stepper motor | Commands and events | Yes | No | Not yet exposed |
| IMU | Events and shared axis types | Yes | No | Not yet exposed |
| RFID | Commands and events | Yes | No | Not yet exposed |
| LiDAR | Payload types; top-level variants disabled | Disabled from module tree | No | Not yet exposed |
| Servo | Payload types; top-level variants disabled | Disabled from module tree | No | Not yet exposed |
| Rangefinder | Payload types; top-level variants disabled | Disabled from module tree | No | Not yet exposed |

## Current limitations

- The Slint application currently covers connection setup, not the previous
  module dashboard or LiDAR visualization.
- Serial input is read and logged but is not yet deserialized into UI state.
- The desktop transport layer does not yet write protocol commands to the serial
  port.
- Wi-Fi and Bluetooth connection paths are placeholders.
- Port enumeration happens only at application startup; hot-plug refresh and
  reconnect behavior are not implemented.
- The firmware's active module selection and hardware pins are hard-coded in
  `Firmware_Templates/src/main.rs`.
- Several protocol and firmware modules are present but are not part of the
  active runtime.
- Root project helpers still contain the pre-refactor `UI_Templates/` path.
- Runtime module UUIDs change on each firmware start.
- The protocol is not versioned and may change during pre-alpha development.
- Automated unit, integration, and hardware-in-the-loop tests are not yet
  included.

## Development checks

Run checks from each crate because there is no root Cargo workspace manifest:

```bash
cd protocol
cargo fmt --all -- --check
cargo check

cd ../UI
cargo fmt --all -- --check
cargo check

cd ../Firmware_Templates
cargo fmt --all -- --check
cargo check
```

Hardware behavior still needs to be validated on a connected ESP32 after a
successful compile check.

## Contributing

When contributing:

1. Keep shared wire types in `protocol/` and consume them from both applications.
2. Preserve the one-JSON-object-per-line framing unless the protocol is migrated
   deliberately on both sides.
3. Keep Slint declarations in `UI/ui/` and native behavior in the Rust bridge or
   transport modules.
4. Document changes to active modules, pins, addresses, calibration, or message
   schemas.
5. Test firmware changes on physical hardware when possible and identify any
   behavior that has only been compile-checked.

## License

No repository-wide license has been added yet. The placeholder license inside
`UI/` does not establish licensing terms for the entire project. Until a project
license is provided, do not assume the repository is open-source licensed.

---

<div align="center">

Built for curious hardware experiments—one module at a time.

**Pinora v0.1.0 · Pre-alpha**

</div>
