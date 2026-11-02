# Development Wi-Fi transport — v0.8.0-alpha

To use Wi-Fi, copy [wifi_config.example.rs](../protocol/src/wifi_config.example.rs) to ignored `protocol/src/wifi_config.rs` (without it, builds use the placeholders and Wi-Fi cannot connect) and set the Wi-Fi SSID, password, laptop IP, listener port, and development token locally. Both the firmware and Tauri backend compile these constants in; rebuild both after changing them. Do not commit the local config or compiled binaries containing credentials. The current firmware selects Bluetooth at startup; select `TransportType::Wifi` in [firmware main](src/main.rs) and rebuild for this Wi-Fi procedure.

The ESP32 connects to DESKTOP_IP:DESKTOP_PORT. Tauri listens on 0.0.0.0 (the `start_wifi_listener` command accepts an optional bind address) using the port entered in the Transport card. That port must match DESKTOP_PORT. The development token gates the TCP connection; a peer has 5 s in total to send it. After it, the desktop runs the protocol v2 device authentication described in the [Bluetooth guide](BLUETOOTH_TRANSPORT.md#pairing-a-desktop-with-the-device-protocol-v2), and a paired device accepts only signed commands. Normal Pinora messages remain newline-delimited JSON.

The TCP listener serves one device at a time; a newly authenticated device replaces the previous connection. Since audit Phase 2B the firmware brings Wi-Fi up in the background, so the board starts (LEDs included) even without an access point, and it reassociates after a drop. It uses bounded queues, caches system/registration messages for reconnect replay, and may drop runtime events under pressure (counted, and reported as one SysLog every 5 s). Live module state is not cached; the desktop requests it with the protocol v1 `Rediscover` command after each connection. Traffic is not encrypted: on the LAN, telemetry is readable and the token can be observed. Commands to a paired device are still protected by per-command MACs, but treat this as a development connection.

## Manual hardware check

1. Put the laptop and ESP32 on the configured Wi-Fi network. Check the laptop's reachable IPv4 address and set DESKTOP_IP accordingly.
2. Set the five values in protocol/src/wifi_config.rs, then rebuild and flash the firmware and launch the Tauri app.
3. In the Transport card, select Wi-Fi, enter the configured port, and click Start Listener.
4. Power or reset the ESP32. The status should move from Listening to Connected.
5. Check that system information and module registrations appear. Use an LED control and confirm the ESP32 executes the command.
6. Click Stop Listener and start it again. The ESP32 should reconnect and send its system and registration messages again.
