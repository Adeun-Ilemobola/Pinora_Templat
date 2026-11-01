# Phase 2B — Transport robustness and authentication (PLAN)

| Field | Value |
|---|---|
| Phase | 2B |
| Status | **Approved** 2026-11-01 |
| Decisions | **Crypto:** RustCrypto `hmac 0.12` + `sha2 0.10` in the shared protocol crate behind an `auth` feature (one implementation for firmware and Tauri). **Provisioning:** pair over USB: the device creates a random key on request and stores it in NVS; the desktop fetches it over the USB serial console only and stores it in the app data directory. **Enforcement:** a keyed device is **read-only** to unauthenticated Bluetooth/Wi-Fi peers (discovery and telemetry flow; commands are rejected with a SysLog). Devices without a key behave as today, with a warning |
| Branch | `phase-2b/transport-auth`, cut from `phase-2a/protocol-v1` @ `b7dfe3a2` (local, not pushed) |
| Linked findings | PIN-TPL-003 (authentication part), PIN-TPL-010, PIN-TPL-013, PIN-TPL-016 |
| Wire format | **Protocol v2, additive.** New system commands, a new `Auth` message family and an optional `auth` field on commands. Existing lines keep their shape; the golden file gains lines and every change is reviewed |

## Design

### Handshake (mutual, HMAC-SHA256, per-device 256-bit key `K`)

| # | Direction | Message |
|---|---|---|
| 1 | desktop → device | `{"id":"SSI","command":{"System":{"AuthBegin":{"client_nonce":"<32 hex>"}}}}` |
| 2 | device → desktop | `{"Auth":{"Challenge":{"device_id","device_nonce","device_proof"}}}`, or `{"Auth":{"Unprovisioned":{"device_id"}}}` if the device has no key |
| 3 | desktop → device | `System(AuthResponse { proof })`, sent only after `device_proof` verifies (the desktop authenticates the device too) |
| 4 | device → desktop | `{"Auth":{"Result":{"authenticated":true\|false,"reason"}}}` |
| 5 | desktop → device | every later command carries `"auth":{"seq":n,"mac":"<64 hex>"}`; `seq` strictly increases per session |

Proofs and keys use domain-separated HMACs: `device_proof = HMAC(K, "pinora-auth-v2/device" ‖ device_id ‖ client_nonce ‖ device_nonce)`, `proof = HMAC(K, "pinora-auth-v2/desktop" ‖ …)`, session key `S = HMAC(K, "pinora-auth-v2/session" ‖ …)`, command `mac = HMAC(S, seq ‖ canonical command JSON without auth)`. Comparisons are constant time (`Mac::verify_slice`). `device_id` is the ESP32 base MAC.

Without per-command MACs, a peer that hijacks an authenticated TCP session (Wi-Fi is plaintext) could inject commands; the MAC and sequence number close that, and replays are rejected.

### Gating on the device (Bluetooth and Wi-Fi sessions)

Always allowed: `AuthBegin`, `AuthResponse`, `Rediscover`. With a key: everything else needs an authenticated session and a valid `auth`. Rejections are answered with a High SysLog from `"log"`. Without a key: everything is accepted, with one Medium SysLog per session. The **USB Serial** image is trusted (physical access) and needs no handshake.

### Provisioning (USB only)

A UART console reader runs in **every** image. It handles `System(ProvisionKey {})` (create the key if missing, reply `Auth(Key { device_id, key })`) and `System(ClearKey {})` (remove, reply `Auth(KeyCleared)`), and in the Serial image it forwards the other commands to the main loop as before. Over Bluetooth or Wi-Fi these commands are rejected. Tauri intercepts `Key` lines from a USB port, stores the key in `<app data>/device-keys.json`, and **never forwards the key to the WebView**. Keys are generated with the hardware RNG (`bootloader_random_enable` while the radio is off).

## Changes

| Step | Change | Findings |
|---|---|---|
| Q1 | **Protocol v2.** `framing` (one `LineFramer`, one limit table: 8 192 B lines everywhere, 512 B auth line), `auth` feature (primitives plus pure `DeviceSession` and `ClientSession` state machines), new wire types, `PROTOCOL_VERSION = 2`. Golden re-blessed; end-to-end handshake, tamper, replay and RFC 4231 tests | 016, 003 |
| Q2 | **Firmware framing and emission.** Bluetooth, Wi-Fi and console use `LineFramer`. Drop counters at every stage, reported as one periodic SysLog instead of a warning per message. `RateLimiter` helper, applied to IMU samples, rangefinder error logs and LiDAR per-step logs. Dead-receiver fallback removed | 010, 016 |
| Q3 | **Firmware Wi-Fi.** Bring-up and reassociation in the background, so boot no longer fails without an AP. Separate reader and writer threads (no 50 ms-per-frame cap) | 013, 010 |
| Q4 | **Firmware authentication.** NVS key store, device id, console reader in every image, `DeviceSession` in the Bluetooth and Wi-Fi workers | 003, 013 |
| Q5 | **Tauri.** `LineFramer` for COM (now bounded) and Wi-Fi. Listener: accept thread, per-peer auth worker with an overall deadline, newest authenticated device replaces the old one, optional bind address. Auth driver (`ClientSession`), key file, command wrapping in `send_data`, key-line interception. IPC (additive): `provision_device_key`, `clear_device_key`, `forget_device_key`, `get_auth_status`. Event (additive): `deviceAuth` | 003, 013, 016 |
| Q6 | **Frontend.** Schemas for the new wire types (golden fixture test), auth status and pairing controls, warnings for unprovisioned devices and failed authentication | 003 |
| Q7 | Docs: contracts C-01, C-02, C-05, C-06, C-07, C-15, C-17/C-19 notes; communication flow; Bluetooth and Wi-Fi guides (pairing workflow); README | — |

Not in scope: TLS or encryption of Wi-Fi traffic (telemetry stays readable on the LAN, which matches "read-only"); OS-keychain storage of desktop keys (would need a new dependency); authentication of the USB Serial transport.

## Compatibility

| Pairing | Result |
|---|---|
| New desktop + device without a key | Works as today; the UI warns "not paired" |
| New desktop + keyed device, key known | Authenticated; commands carry MACs |
| New desktop + keyed device, key unknown | Read-only; the UI says to pair over USB |
| Old desktop + keyed device | Read-only (it cannot authenticate) |
| Old desktop + device without a key | Works as today |
| New desktop + pre-v2 firmware | `AuthBegin` is logged as invalid by the firmware; the UI reports "firmware does not support authentication" |

## Verification

| # | Check | Pass criterion |
|---|---|---|
| B2-V1 | Protocol | Golden diff limited to new lines; framing, primitive, handshake, tamper, replay, wrong-key, RFC 4231 tests pass |
| B2-V2 | Host firmware | Rate limiter, drop accounting and console routing tests; existing tests pass |
| B2-V3 | Tauri | Key file, client driver, listener (slow peer within deadline, malformed peer, replacement) tests; clippy |
| B2-V4 | Frontend | Fixture test covers the new families; `tsc`; Vite build; `bun test` |
| B2-V5 | Builds | Firmware debug and release; lockfile diff contains only the new crates; rustfmt |
| B2-V6 | Mutation | Skipping the MAC check, accepting an out-of-order `seq`, or skipping `device_proof` verification makes tests fail |
| B2-V7 | Hardware (yours) | Pair over USB; Bluetooth commands work; a second, unpaired desktop is read-only; Wi-Fi boot without an AP still starts the LEDs |

## Recovery

Revert the 2B commits, or `git switch phase-2a/protocol-v1`. Keys stored in NVS stay but are ignored by v1 firmware; `ClearKey` over USB removes them under v2.
