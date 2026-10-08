# Development Wi-Fi transport — v0.8.0-alpha

Copy [wifi_config.example.rs](../protocol/src/wifi_config.example.rs) to ignored `protocol/src/wifi_config.rs` and set the Wi-Fi SSID, password, laptop IP, listener port, and development token locally. Both the firmware and Tauri backend compile these constants in; rebuild both after changing them. Do not commit the local config or compiled binaries containing credentials. The current firmware selects Bluetooth at startup; select `TransportType::Wifi` in [firmware main](src/main.rs) and rebuild for this Wi-Fi procedure.

The ESP32 connects to DESKTOP_IP:DESKTOP_PORT. Tauri listens on 0.0.0.0 using the port entered in the Transport card. That port must match DESKTOP_PORT. The development token is used only for the connection handshake; normal Pinora messages remain newline-delimited JSON.

The TCP listener accepts one authenticated device at a time. Firmware uses bounded queues, caches system/registration messages for reconnect replay, and may drop runtime events under pressure. It does not replay live module state. The listener binds all interfaces and the token is sent without transport encryption, so this is a development-only connection.

## Manual hardware check

1. Put the laptop and ESP32 on the configured Wi-Fi network. Check the laptop's reachable IPv4 address and set DESKTOP_IP accordingly.
2. Set the five values in protocol/src/wifi_config.rs, then rebuild and flash the firmware and launch the Tauri app.
3. In the Transport card, select Wi-Fi, enter the configured port, and click Start Listener.
4. Power or reset the ESP32. The status should move from Listening to Connected.
5. Check that system information and module registrations appear. Use an LED control and confirm the ESP32 executes the command.
6. Click Stop Listener and start it again. The ESP32 should reconnect and send its system and registration messages again.
