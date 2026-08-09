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