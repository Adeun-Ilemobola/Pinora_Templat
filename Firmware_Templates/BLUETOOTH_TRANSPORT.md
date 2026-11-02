# Bluetooth Classic SPP — v0.8.0-alpha

The current firmware source selects Bluetooth on the **classic ESP32** with ESP-IDF 5.5.3 and the supported `esp-1.93` compiler. Windows pairing authenticated and the outgoing SPP COM port was identified. The maintainer reports Bluetooth operation; Codex has not independently verified every command, event, and reconnect path on hardware. See the [master README](../README.md) for the alpha policy and the [architecture review](../docs/architecture-audit.md).

## Why SPP

ESP32 Classic SPP exposes a serial service that Windows can map to a paired virtual COM port. The desktop reuses its existing `serialport` reader and `send_data` path, with no additional Bluetooth library or Bluetooth-specific wire protocol. ESP-IDF's Bluedroid Classic SPP and `esp-idf-svc` wrappers are used on firmware.

The firmware's [startup selection](src/main.rs) selects `TransportType::Bluetooth`. Serial and Wi-Fi remain source-selectable alternatives; it runs one transport per image. [sdkconfig.defaults](sdkconfig.defaults) enables Classic SPP and [shared Bluetooth settings](../protocol/src/bluetooth_config.rs) define the advertised name, service name, 8 KiB message limit and SPP write chunk size. No static pairing PIN or Bluetooth token is stored in source.

For a fresh checkout, copy the [Wi-Fi config example](../protocol/src/wifi_config.example.rs) to ignored `protocol/src/wifi_config.rs` before building: the shared protocol declares that module even when Bluetooth is selected. Placeholder values are suitable only when Wi-Fi is not used.

The board advertises `Pinora Dev ESP32` and offers the `Pinora SPP` service. The server requests authenticated/encrypted SPP and uses Windows Secure Simple Pairing with the board's no-input/no-display capability. This is the development-grade Just Works association; it has no numeric-comparison MITM protection. Legacy fixed-PIN requests are rejected. **New devices can pair only during the first 120 s after the SPP service starts**; after that the board is connectable but not discoverable and rejects new pairings, while already-bonded devices still connect (audit Phase 1C, PIN-TPL-003). Power-cycle the board to open a new pairing window. Windows pairing only opens the radio link: a device paired in the window can send commands unless the board has an application key (see **Pairing a desktop with the device** below, audit Phase 2B). Until then, do not use this link for sensitive commands on an untrusted radio environment.

## Windows pairing and connection

1. Boot firmware built with Bluetooth selected and watch the USB serial console for `Pinora SPP service is discoverable; pairing accepted for 120 s`. Complete step 2 within that window; afterwards the console logs `Bluetooth pairing window closed`. The USB CP210x port is for console/flashing; discover its current number rather than assuming a prior assignment.
2. Open **Windows Settings → Bluetooth & devices → Add device → Bluetooth**. Select **Pinora Dev ESP32** and complete the Windows pairing prompt. If it is absent, confirm Bluetooth is enabled on the PC and that the firmware reached SPP startup.
3. Open **Device Manager → Ports (COM & LPT)** or **Bluetooth → More Bluetooth settings → COM Ports**. Identify the *outgoing* virtual COM port for Pinora's SPP service. A paired SPP port is distinct from the CP210x USB port.
4. Run the native Tauri desktop (`cd UI; bun run tauri dev`). In the Transport card select **Bluetooth**, click **Refresh ports**, select the paired *outgoing* Bluetooth COM port, and click **Connect**. Tauri checks Windows device metadata because `serialport` may classify a Bluetooth COM port as `Unknown`; it does not treat every unknown port as Bluetooth. Enumeration failures appear in the card. If no Bluetooth port appears, inspect pairing and the COM Ports tab. Pinora does not create a COM port or install drivers.
5. On a working connection, system information and the three active LED registrations should arrive. Use an LED control, verify the correct physical output, and check for a returned brightness event. Disconnect, then connect again to check registration/system replay; the LED cards should show the brightness you set, not 0 % (protocol v1 rediscovery).

The virtual COM port's baud setting is ignored by Windows SPP. Pinora opens it at 115200 solely because `serialport` requires a value. The backend does not toggle USB DTR/RTS for Bluetooth. A read error reports a lost Bluetooth connection; reconnect from the card after clearing the error.

## Pairing a desktop with the device (protocol v2)

Since audit Phase 2B the device can require **application-level authentication** on Bluetooth and Wi-Fi (PIN-TPL-003). It is opt-in per device:

1. Connect the board's USB port and, in the Transport card, select **Serial** and connect to the USB (CP210x) port. Any firmware image works; the USB console accepts pairing in every image.
2. In **Device security**, click **Pair this device**. The device creates a random 256-bit key (stored in its NVS) and answers on the USB console only. The desktop stores it in `device-keys.json` in its app data directory; the key never reaches the web view, and pairing is never accepted over Bluetooth or Wi-Fi.
3. Disconnect, then connect over Bluetooth or Wi-Fi. The card should say **Authenticated**: the desktop and device prove the key to each other (HMAC-SHA256 challenge-response) and every command is then signed with a per-session key and a sequence number, so tampered or replayed commands are rejected.

Once a device has a key, a desktop that cannot authenticate is **read-only**: it still sees system information, registrations and telemetry, but every command is refused with a High SysLog. A device without a key accepts commands as before and the card warns "Device not paired". **Clear device key** (over USB) removes the key on the device; forgetting a device on the desktop does not. Firmware older than protocol v2 cannot authenticate, and the card says so.

## Message flow and limits

SPP callback data is copied into a bounded worker queue. The worker frames lines with the shared protocol framer (8 KiB on every transport since Phase 2B), accepts multiple JSON lines in one callback, passes each through the connection's authentication session, and sends permitted commands to the same main-loop UUID/module command handler used by Serial and Wi-Fi. Module output follows the existing `TransportEmitter` path and serializes the same `ProtocolMessage` envelopes. SPP writes are chunked and wait for completion and congestion clearance. On queue overflow, the firmware disconnects to resynchronize framing.

The worker caches the latest system message and registrations for replay after reconnection. Runtime module events are not cached. Current module state (such as LED brightness) is re-sent when the desktop requests rediscovery, which it does after every connect (protocol v1). Callback queues, outgoing frames, incoming commands and (since Phase 2B) the desktop COM line buffer are bounded; the frontend log history is not. Dropped telemetry is counted and reported as one SysLog every 5 s. No acknowledgement, automatic transport failover, or simultaneous multi-transport routing exists.

The active firmware modules are three PWM LEDs. Experimental LiDAR, servos, and other compiled module families are not active in this image. Maintainer-reported operation does not establish an automated command-by-command hardware validation record.
