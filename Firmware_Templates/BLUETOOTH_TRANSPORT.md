# Bluetooth Classic SPP — v0.8.0-alpha

The current firmware source selects Bluetooth on the **classic ESP32** with ESP-IDF 5.5.3 and the supported `esp-1.93` compiler. Windows pairing authenticated and the outgoing SPP COM port was identified. The maintainer reports Bluetooth operation; Codex has not independently verified every command, event, and reconnect path on hardware. See the [master README](../README.md) for the alpha policy and the [architecture review](../docs/architecture-audit.md).

## Why SPP

ESP32 Classic SPP exposes a serial service that Windows can map to a paired virtual COM port. The desktop reuses its existing `serialport` reader and `send_data` path, with no additional Bluetooth library or Bluetooth-specific wire protocol. ESP-IDF's Bluedroid Classic SPP and `esp-idf-svc` wrappers are used on firmware.

The firmware's [startup selection](src/main.rs) selects `TransportType::Bluetooth`. Serial and Wi-Fi remain source-selectable alternatives; it runs one transport per image. [sdkconfig.defaults](sdkconfig.defaults) enables Classic SPP and [shared Bluetooth settings](../protocol/src/bluetooth_config.rs) define the advertised name, service name, 8 KiB message limit and SPP write chunk size. No static pairing PIN or Bluetooth token is stored in source.

For a fresh checkout, copy the [Wi-Fi config example](../protocol/src/wifi_config.example.rs) to ignored `protocol/src/wifi_config.rs` before building: the shared protocol declares that module even when Bluetooth is selected. Placeholder values are suitable only when Wi-Fi is not used.

The board advertises `Pinora Dev ESP32` and offers the `Pinora SPP` service. The server requests authenticated/encrypted SPP and uses Windows Secure Simple Pairing with the board's no-input/no-display capability. This is the development-grade Just Works association; it has no numeric-comparison MITM protection. Legacy fixed-PIN requests are rejected. Do not use this link for sensitive commands on an untrusted radio environment.

## Windows pairing and connection

1. Boot firmware built with Bluetooth selected and watch the USB serial console for `Pinora SPP service is discoverable`. The USB CP210x port is for console/flashing; discover its current number rather than assuming a prior assignment.
2. Open **Windows Settings → Bluetooth & devices → Add device → Bluetooth**. Select **Pinora Dev ESP32** and complete the Windows pairing prompt. If it is absent, confirm Bluetooth is enabled on the PC and that the firmware reached SPP startup.
3. Open **Device Manager → Ports (COM & LPT)** or **Bluetooth → More Bluetooth settings → COM Ports**. Identify the *outgoing* virtual COM port for Pinora's SPP service. A paired SPP port is distinct from the CP210x USB port.
4. Run the native Tauri desktop (`cd UI; bun run tauri dev`). In the Transport card select **Bluetooth**, click **Refresh ports**, select the paired *outgoing* Bluetooth COM port, and click **Connect**. Tauri checks Windows device metadata because `serialport` may classify a Bluetooth COM port as `Unknown`; it does not treat every unknown port as Bluetooth. Enumeration failures appear in the card. If no Bluetooth port appears, inspect pairing and the COM Ports tab. Pinora does not create a COM port or install drivers.
5. On a working connection, system information and the three active LED registrations should arrive. Use an LED control, verify the correct physical output, and check for a returned brightness event. Disconnect, then connect again to check registration/system replay.

The virtual COM port's baud setting is ignored by Windows SPP. Pinora opens it at 115200 solely because `serialport` requires a value. The backend does not toggle USB DTR/RTS for Bluetooth. A read error reports a lost Bluetooth connection; reconnect from the card after clearing the error.

## Message flow and limits

SPP callback data is copied into a bounded worker queue. The worker collects partial reads until newline, accepts multiple JSON lines in one callback, rejects commands beyond 8 KiB, deserializes the existing Rust `IncomingCommand`, and sends it to the same main-loop UUID/module command handler used by Serial and Wi-Fi. Module output follows the existing `TransportEmiter` path and serializes the same `ProtocolMessage` envelopes. SPP writes are chunked and wait for completion and congestion clearance. On queue overflow, the firmware disconnects to resynchronize framing.

The worker caches the latest system message and registrations for replay after reconnection. Runtime module events and LED brightness state are not replayed; a fresh command/event is needed to observe current brightness. Callback queues, outgoing frames, and incoming commands are bounded, but the desktop COM line buffer and frontend log history are not. No application-level acknowledgement, sequencing, automatic transport failover, or simultaneous multi-transport routing exists.

The active firmware modules are three PWM LEDs. Experimental LiDAR, servos, and other compiled module families are not active in this image. Maintainer-reported operation does not establish an automated command-by-command hardware validation record.
