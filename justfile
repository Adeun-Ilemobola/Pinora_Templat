flash:
    cd Firmware_Templates && cargo +esp-1.93 espflash flash --release --monitor

frontend:
    cd UI_Templates && bun run dev:hmr
