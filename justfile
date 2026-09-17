[windows]
set shell := ["pwsh", "-NoLogo", "-Command"]

default:
    just --list


# ============================================================
# Development
# ============================================================

# Check the Rust projects, ensure UI dependencies exist,
# then launch the Tauri development application.
dev: check-protocol check-firmware
    cd UI && bun install --frozen-lockfile
    cd UI && bun tauri dev


# Run only the UI in development mode.
ui:
    cd UI && bun install --frozen-lockfile
    cd UI && bun tauri dev


# Run the UI using an optimized Rust/Tauri build.
ui-release:
    cd UI && bun install --frozen-lockfile
    cd UI && bun tauri dev --release


# ============================================================
# Build
# ============================================================

# Build the entire project.
build-all: build-protocol build-ui build-firmware

build-protocol:
    cd protocol && cargo build

build-ui:
    cd UI && bun install --frozen-lockfile
    cd UI && bun tauri build --no-bundle

build-firmware:
    cd Firmware_Templates && cargo +esp-1.93 build


# ============================================================
# Check
# ============================================================

# Validate everything without producing release artifacts.
check-all: check-protocol check-ui check-firmware

check-protocol:
    cd protocol && cargo check

check-ui:
    cd UI && bun install --frozen-lockfile
    cd UI && bun run build
    cd UI && cargo check --manifest-path src-tauri/Cargo.toml

check-firmware:
    cd Firmware_Templates && cargo +esp-1.93 check


# ============================================================
# Firmware
# ============================================================

# Build, flash, and monitor the ESP32.
# DO NOT MODIFY: known-good firmware flash command.
flash:
    cd Firmware_Templates && cargo +esp-1.93 espflash flash --release --monitor


# ============================================================
# Clean
# ============================================================

clean: clean-protocol clean-ui clean-firmware

clean-protocol:
    cd protocol && cargo clean

clean-ui:
    cd UI && cargo clean --manifest-path src-tauri/Cargo.toml
    if (Test-Path "UI/dist") { Remove-Item -Recurse -Force "UI/dist" }

clean-firmware:
    cd Firmware_Templates && cargo +esp-1.93 clean