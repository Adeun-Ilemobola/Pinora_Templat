flash:
    cd Firmware_Templates && cargo espflash flash --release --monitor

frontend:
    cd UI_Templates && bun run dev:hmr
