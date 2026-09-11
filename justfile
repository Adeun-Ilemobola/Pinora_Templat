[windows]
set shell := ["pwsh", "-NoLogo", "-Command"]

default:
    just --list

# Run the Slint desktop application
ui:
    cd UI && cargo run

# Run an optimized Slint build
ui-release:
    cd UI && cargo run --release

# Build everything
build-all: build-protocol build-ui build-firmware

build-protocol:
    cd protocol && cargo build

build-ui:
    cd UI && cargo build

build-firmware:
    cd Firmware_Templates && cargo +esp-1.93 build

# Check everything without producing final binaries
check-all: check-protocol check-ui check-firmware

check-protocol:
    cd protocol && cargo check

check-ui:
    cd UI && cargo check

check-firmware:
    cd Firmware_Templates && cargo +esp-1.93 check

# Build, flash, and monitor the ESP32
flash:
    cd Firmware_Templates && cargo +esp-1.93 espflash flash --release --monitor

frontend:
    cd UI_Templates && bun run dev:hmr

buildAll:
    just buildUI && just buildFirmware

buildUI:
    cd UI_Templates && bun i

buildFirmware:
    cd Firmware_Templates && cargo build