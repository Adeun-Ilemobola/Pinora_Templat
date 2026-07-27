<div align="center">

# Pinora

### An Electrobun desktop control surface and modular Rust firmware for ESP32 projects

Connect. Discover. Monitor. Control.

![Version](https://img.shields.io/badge/version-0.1.0-7c3aed?style=for-the-badge)
![Status](https://img.shields.io/badge/status-PRE--ALPHA-f97316?style=for-the-badge)
[![Electrobun](https://img.shields.io/badge/Electrobun-1.18-7c3aed?style=for-the-badge)](https://electrobun.dev/)
[![React](https://img.shields.io/badge/React-18-149ECA?style=for-the-badge&logo=react&logoColor=white)](https://react.dev/)
[![Bun](https://img.shields.io/badge/Bun-runtime-000000?style=for-the-badge&logo=bun&logoColor=white)](https://bun.sh/)
[![ESP32](https://img.shields.io/badge/target-ESP32-E7352C?style=for-the-badge&logo=espressif&logoColor=white)](https://www.espressif.com/en/products/socs/esp32)
[![Rust firmware](https://img.shields.io/badge/firmware-Rust-000000?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)

> [!WARNING]
> **Pinora v0.1.0 is a pre-alpha prototype.** Its architecture, protocol, APIs,
> wiring, and module behavior may change without notice. It is intended for
> experimentation and active development, not production or safety-critical use.

</div>

---

## What is Pinora?

Pinora is an experimental system for discovering, monitoring, and controlling
hardware modules connected to an ESP32. It consists of two cooperating projects:

- An Electrobun desktop application built with React, TypeScript, and Bun
- Modular ESP32 firmware written in Rust

The firmware registers its available modules at runtime and publishes their
events over a serial connection. The desktop application validates those
messages, builds a live interface from the registrations, keeps module state in
sync, and sends typed commands back to the ESP32.

The current firmware combines two servos and a VL53L1X time-of-flight sensor into
a two-axis LiDAR scanner. The desktop application provides device controls,
serial-port selection, and an interactive LiDAR visualization.

## Project status

| | State |
|---|---|
| **Release** | `v0.1.0` |
| **Maturity** | Pre-alpha |
| **Firmware target** | ESP32 / `xtensa-esp32-espidf` |
| **ESP-IDF** | `v5.5.3` |
| **Firmware edition** | Rust 2021 |
| **Desktop stack** | Electrobun 1.18, Bun, React 18, TypeScript, Vite |
| **Device interface** | Newline-delimited JSON over UART |
| **Default baud rate** | `115200` |

## System architecture

```mermaid
flowchart LR
    UI["React webview"] <--> STATE["Zustand module store"]
    UI <-->|"Typed Electrobun RPC"| BUN["Electrobun Bun process"]
    BUN --> VALIDATE["Zod protocol validation"]
    BUN <-->|"JSON lines over UART"| ESP["ESP32 firmware runtime"]

    ESP --> REGISTRY["Module registry"]
    ESP --> HARDWARE["Shared hardware context"]
    REGISTRY --> LIDAR["LiDAR module"]
    LIDAR --> SERVOX["X-axis servo"]
    LIDAR --> SERVOY["Y-axis servo"]
    LIDAR --> RANGE["VL53L1X rangefinder"]
    HARDWARE --> I2C["I²C bus"]
    I2C --> PWM["PCA9685"]
    I2C --> RANGE
    PWM --> SERVOX
    PWM --> SERVOY
```

The system is divided into four main layers:

1. **Firmware modules** own hardware behavior, receive typed commands, and emit
   registrations and events.
2. **Serial protocol** carries one JSON object per line between the firmware and
   desktop application.
3. **Electrobun Bun process** owns native serial-port access, validates incoming
   messages with Zod, and exposes typed RPC requests and messages to the webview.
4. **React webview** receives validated messages over Electrobun RPC, maintains
   module state with Zustand, and renders controls and visualizations.

The desktop boundary is entirely TypeScript: Electrobun runs the privileged Bun
process and hosts the React UI in a webview. The Bun process handles
`bun-serialport`; hardware access never runs inside the webview. Shared RPC and
desktop protocol types live under `UI_Templates/src/shared/`.

## Current capabilities

- Discover serial ports and connect at `115200` baud
- Register modules dynamically when the ESP32 announces them
- Track runtime UUIDs, readable lookup IDs, and parent-child relationships
- Validate incoming messages before adding them to application state
- Forward validated serial messages from the Bun process to the React webview
- Display live module state and send hardware commands
- Control two servos through a PCA9685 PWM controller
- Read distance measurements from a VL53L1X sensor
- Run a two-axis LiDAR scan over a selected region of interest
- Visualize LiDAR distance data on an interactive 181 × 181 canvas

### Module support

| Module | Firmware | Active in firmware | Desktop support |
|---|:---:|:---:|:---:|
| LiDAR | ✅ | ✅ | Registration, events, scan controls, ROI, movement, heatmap |
| Rangefinder | ✅ | ✅, as LiDAR child | Registration, events, and controls |
| Servo | ✅ | ✅, as LiDAR children | Registration, events, and angle control |
| LED | ✅ | ❌ | Registration, events, brightness, and toggle controls |
| Button | ✅ | ❌ | Registration and read-only press state |
| LED cluster | Partial | ❌ | Identifier only |
| IMU | Planned | ❌ | Identifier only |
| System log | ✅ | ✅ | Schema validation only; events are not yet stored |

## Folder structure

```text
.
├── UI_Templates/                 # Desktop application
│   ├── src/
│   │   ├── bun/
│   │   │   └── index.ts           # Electrobun main process and serial I/O
│   │   ├── mainview/               # React webview
│   │   │   ├── Modules/            # Module schemas and controls
│   │   │   ├── components/         # Reusable UI
│   │   │   ├── electrobun.ts       # Webview-side RPC handlers
│   │   │   └── main.tsx            # React entry point
│   │   ├── Runtime/
│   │   │   └── ModuleStore.ts      # Zustand module state
│   │   └── shared/
│   │       ├── Protocol/            # TypeScript wire schemas
│   │       └── rpc.ts               # Shared typed RPC contract
│   ├── electrobun.config.ts
│   ├── package.json
│   ├── vite.config.ts
│   └── bun.lock
├── Firmware_Templates/            # ESP32 firmware
│   ├── .cargo/config.toml          # ESP target, linker, and runner
│   ├── .github/workflows/          # Firmware CI checks
│   ├── src/
│   │   ├── core/                   # Module and hardware foundations
│   │   ├── module/                 # Hardware module implementations
│   │   ├── protocol/               # Rust wire-protocol definitions
│   │   ├── utilities/              # Logging and math helpers
│   │   └── main.rs                 # Firmware setup and main loop
│   ├── Cargo.toml
│   ├── rust-toolchain.toml
│   └── sdkconfig.defaults
└── README.md
```

## Getting started

### Prerequisites

For the desktop application, install:

- [Bun](https://bun.sh/)
- A USB serial driver for the ESP32 board, if required

For the firmware, install the ESP Rust development environment and ensure the
following commands are available:

- `cargo`
- The Espressif `esp` Rust toolchain
- `ldproxy`
- `espflash`

The firmware selects the `esp` toolchain automatically and targets
`xtensa-esp32-espidf`.

### Build and run the firmware

```bash
cd Firmware_Templates
cargo build
cargo run --release
```

The configured Cargo runner uses `espflash flash --monitor`, so
`cargo run --release` builds the firmware, flashes the connected ESP32, and opens
the serial monitor.

To create a size-optimized build without flashing:

```bash
cd Firmware_Templates
cargo build --release
```

### Run the desktop application

```bash
cd UI_Templates
bun install
bun run dev:hmr
```

Then:

1. Select the ESP32 from the available serial ports.
2. Connect and wait for module registrations.
3. Use the generated module controls to interact with the hardware.

The desktop serial runtime currently uses a fixed baud rate of `115200`.

For Electrobun watch mode with bundled Vite assets instead of HMR:

```bash
cd UI_Templates
bun run dev
```

Serial features run in Electrobun's Bun process and are reached from the React
webview through typed RPC. They do not work when the Vite UI is opened as a
standalone browser tab.

### Build the desktop application

```bash
cd UI_Templates
bun run build:canary
```

This runs the Vite production build and then creates an Electrobun canary build.

## Hardware

The active firmware configuration expects:

- An ESP32 development board
- A PCA9685 16-channel PWM controller
- Two compatible hobby servos
- A VL53L1X time-of-flight distance sensor
- A shared I²C bus using GPIO 21 for SDA and GPIO 22 for SCL at 400 kHz
- Servo X on PCA9685 channel `C0`
- Servo Y on PCA9685 channel `C1`

> [!WARNING]
> Do not power servos directly from the ESP32's 3.3 V pin. Use a suitable
> external supply, connect the grounds, verify voltage requirements, and test
> with the mechanism unloaded. The current firmware moves both servos during
> initialization.

The current servo defaults are:

| Setting | Value |
|---|---:|
| Physical angle range | 0° to 180° |
| Logical pivot range | -90° to +90° |
| Center offset | 90° |
| Pulse range | 500–2500 µs |

These values are currently defined in
`Firmware_Templates/src/module/lidar.rs` and must be calibrated for the physical
assembly.

## Serial protocol

Pinora uses one newline-terminated JSON object per message:

- Commands travel from the desktop application to the ESP32.
- Registrations, module events, and system logs travel from the ESP32 to the
  desktop application.
- Each module receives a runtime UUID when the firmware starts.

The Rust definitions in `Firmware_Templates/src/protocol/` and TypeScript/Zod
definitions in `UI_Templates/src/shared/Protocol/` must remain aligned.

### Register a module

```json
{
  "type": "Registration",
  "payload": {
    "id": "generated-runtime-uuid",
    "module_type": "Servo",
    "lool_up_id": "servo_x",
    "parent_id": "parent-lidar-uuid"
  }
}
```

> [!NOTE]
> `lool_up_id` is the current wire-format field name. Its spelling is retained
> for compatibility during pre-alpha development.

### Send a module event

```json
{
  "type": "ModuleEvent",
  "payload": {
    "module_type": "Led",
    "event": {
      "event_type": "Brightness",
      "id": "led-runtime-uuid",
      "level": 80
    }
  }
}
```

### Send commands to the firmware

Start a LiDAR scan:

```json
{"id":"lidar-runtime-uuid","module_type":"Lidar","payload":{"command":"StartScan"}}
```

Change the scan step:

```json
{"id":"lidar-runtime-uuid","module_type":"Lidar","payload":{"command":"SetStep","step":5}}
```

Move to a position:

```json
{"id":"lidar-runtime-uuid","module_type":"Lidar","payload":{"command":"MovePos","p":{"x":15,"y":-10}}}
```

Set a region of interest:

```json
{"id":"lidar-runtime-uuid","module_type":"Lidar","payload":{"command":"Roi","min":{"x":45,"y":-30},"max":{"x":-45,"y":30}}}
```

## Current limitations

- The serial protocol is not versioned and may change.
- Firmware pins, PWM channels, servo calibration, and scan defaults are
  hard-coded.
- Module UUIDs change after every restart.
- The firmware currently activates only the composite LiDAR path.
- UART is the only firmware transport.
- The scan loop advances from firmware ticks rather than confirmed servo
  settling.
- Reconnect and device hot-plug behavior are basic.
- Protocol definitions are duplicated between the Rust firmware and TypeScript
  desktop application.
- Some low-level firmware operations still need safer error recovery.
- Automated unit, integration, and hardware-in-the-loop tests are not yet
  included.
- Desktop packaging has not been validated on every supported platform.

## Development checks

Run the desktop checks:

```bash
cd UI_Templates
bun run build:canary
```

Run the firmware checks:

```bash
cd Firmware_Templates
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo build --release
```

## Contributing

When contributing:

1. Keep the Rust and TypeScript protocol definitions aligned.
2. Preserve the one-JSON-object-per-line wire format.
3. Validate incoming serial data before adding it to application state.
4. Document changes to pins, addresses, calibration, or message schemas.
5. Test changes against physical hardware when possible.
6. Clearly identify behavior that has not been hardware-tested.

## License

No license has been added yet. Until one is provided, the source remains under
the copyright of its author and should not be assumed to be open-source
licensed.

---

<div align="center">

Built for curious hardware experiments—one module at a time.

**Pinora v0.1.0 · Pre-alpha**

</div>
