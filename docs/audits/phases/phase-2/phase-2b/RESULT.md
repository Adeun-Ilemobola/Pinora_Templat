# Phase 2B — Transport robustness and authentication (RESULT)

| Field | Value |
|---|---|
| Phase | 2B |
| Status | **Completed — Verified**: host checks B2-V1…B2-V6 pass, and the Bluetooth part of the hardware check B2-V7 passed on your ESP32 (2026-11-02). The Wi-Fi hardware check was not run (it needs the Wi-Fi image and local network credentials) |
| Date | 2026-11-01 … 2026-11-02 |
| Branch | `phase-2b/transport-auth`, cut from `phase-2a/protocol-v1` @ `b7dfe3a2` (local, not pushed; `main` unchanged at `5b5c8ed4`) |
| Decisions applied | RustCrypto `hmac`/`sha2` in the protocol crate (`auth` feature); pairing over USB; keyed devices read-only to unauthenticated peers |
| Hardware authorization | On 2026-11-02 you authorized testing on the attached board (LEDs on GPIO12/14/27) and all remaining phases. The board was flashed with the 2B release image using the project's own bootloader and partition table, as `just flash` does |
| Findings | PIN-TPL-003, -010, -013, -016 → **Resolved** |
| Wire format | **Protocol v2, additive.** `msg.system` reports 2; 11 new `tauri.*` golden lines; every other line unchanged |

## 1. Commits

| Step | Commit | Change |
|---|---|---|
| Plan | `ea29668b` | Phase 2B plan |
| Q1 | `b126a1d7` | Protocol v2: `framing` (one `LineFramer` and limit table), `auth` (wire types; with the feature: HMAC primitives, `DeviceSession`, `ClientSession`), new system commands and `ProtocolMessage::Auth`; golden re-blessed |
| Q2–Q3 | `dcde0423` | Firmware: shared framing on every transport, drop counters with a 5 s SysLog report, `RateLimiter` (IMU 20/s coalesced, rangefinder errors and LiDAR step logs 1/s), Wi-Fi background bring-up and reassociation, Wi-Fi reader/writer split. One commit because both steps rewrite `wifi.rs` |
| Q4 | `e07262e9` | Firmware authentication: NVS key store, device id, USB console in every image, `auth_gate`, sessions in the Bluetooth and Wi-Fi workers |
| Q5a | `353de798` | Desktop building blocks: `Auth` lines are a classifier variant with no WebView event; key file; `AuthLink`; `EventSink` |
| Q5b | `d221b6df` | Desktop transports: COM authenticates and signs, Wi-Fi listener with per-peer deadline and replacement, new IPC commands, `deviceAuth` event |
| Q6 | `3b616771` | Frontend: `deviceAuth` schema, store, Device security card with USB pairing |
| Q7 | *(docs commit)* | Contracts, flow, guides, register, RESULT; stronger trickle test (M5); `tests/hardware-probe` and `just hw-test` |

## 2. Golden wire diff (reviewed)

| Line | Change |
|---|---|
| `msg.system` | `"protocol_version":2` (was 1) |
| `tauri.cmd.auth_begin`, `tauri.cmd.auth_response`, `tauri.cmd.provision_key`, `tauri.cmd.clear_key` | **New** system commands |
| `tauri.signed_command` | **New**: `{"seq":7,"mac":"…","signed":"<command JSON>"}` |
| `tauri.auth.challenge`, `.unprovisioned`, `.result.ok`, `.result.failed`, `.key`, `.key_cleared` | **New** `Auth` messages |

Every other line is byte-identical. The frontend fixture test checks that all 11 `tauri.*` lines are rejected by the WebView schemas, because they never reach the WebView.

## 3. Verification

| # | Check | Result |
|---|---|---|
| B2-V1 | Protocol | ✅ **31 pass** with `--all-features` (18 auth: RFC 4231 vectors, full handshake, f32 round trip, tamper, replay and reordering, read-only, wrong key both ways, forged proof, stale response, unprovisioned, provisioning refused even inside a signed command, key change ends the session, malformed nonce; 7 framing), **13 pass** without features. Golden diff as in §2 |
| B2-V2 | Host firmware | ✅ **97 pass in debug and release** (was 79): drop counters (per stage, threads, reset), rate limiter (interval, flood cut to 20/s, saturation), emitter drop counting, `auth_gate` end to end with `ClientSession`, console routing per image |
| B2-V3 | Tauri | ✅ **19 pass** (was 5): key file (persist, corrupt file, id validation), `AuthLink` (Bluetooth handshake and signing, no key, USB pairing, keys over radio never stored, reserved commands refused), classifier never forwards `Auth`, listener over real localhost TCP (full handshake with signed commands, trickling peer cut at the overall deadline, wrong token, replacement, unprovisioned). Clippy: only the 2 baseline warnings |
| B2-V4 | Frontend | ✅ `bun test` **144 pass** (was 137); `tsc` clean; Vite build passes |
| B2-V5 | Builds | ✅ Firmware debug and release build with `--frozen`, warnings unchanged at 6. Lockfiles only gain crates (firmware: `hmac`, `sha2` and 7 dependencies, one entry re-qualified as `cpufeatures 0.3.1`; Tauri: `hmac`, `subtle`, `getrandom` made direct; host crate: same crypto crates, `serde_json` pinned to the firmware's 1.0.151, which moved the wasm-only `js-sys`/`wasm-bindgen` one patch down). rustfmt clean on 4 crates. Host-crate clippy: the 3 pre-existing warnings |
| B2-V6 | Mutation | ✅ M1 skip the command MAC check → 1 failure. M2 accept a stale `seq` → 1. M3 skip the device-proof check → 1. M4 forward `Auth` lines to the WebView → 3. M5 per-read instead of overall deadline → **survived at first**; the trickle test was strengthened (the peer now trickles for five deadlines) and then fails under the mutation. Files restored |
| B2-V7 | Hardware (your board) | ✅ Bluetooth: **23/23** scenario checks and **5/5** persistence checks (§5). ⏳ Wi-Fi: not run |

ELF sizes (text / data / bss):

| Image | 2A | 2B | Δ |
|---|---|---|---|
| Release | 1 128 086 / 251 982 / 42 539 | 1 026 635 / 232 934 / 31 467 | −101 451 / −19 048 / −11 072 |
| Debug | 1 613 532 / 504 593 / 45 636 | 1 657 148 / 519 041 / 45 636 | +43 616 / +14 448 / 0 |

The release image **shrank** although code was added. Probable cause: link-time optimization now removes the unused Wi-Fi stack from the Bluetooth image. Evidence: the 2B release image contains none of `Wi-Fi connecting`, `wifi-transport`, `TCP connecting` or `wpa_supplicant`, while the 2B debug image contains all four, and the new code is present in release (`pinora-auth-v2/device`, `usb-console`, `Telemetry dropped under load`). The 2A release image was not kept, so this is not proven by a side-by-side comparison. The app now uses 1 259 680 of the 1 536 000-byte factory partition (82%); the debug image would not fit.

## 4. What changed for you

- **Pairing is opt-in.** A device stays open, as before, until you pair it over USB with **Pair this device**. After that, a desktop that cannot authenticate can only watch.
- **Commands from the WebView are now validated on every transport.** Before 2B, commands sent over COM were unvalidated. The WebView can no longer send pairing or handshake commands.
- **Bounded lines everywhere.** The desktop COM reader now drops a line over 8 KiB; before, it had no limit.
- **Wi-Fi boot no longer depends on the access point**, and the listener tolerates slow peers and reconnecting devices.

## 5. Hardware run (2026-11-02, ESP32 rev v3.1, MAC 3c:8a:1f:5d:75:ac, LEDs on GPIO12/14/27)

Flashed: the 2B release image, the project's ESP-IDF v5.5.3 bootloader and its partition table (the first attempt used espflash's bundled bootloader and default table, and was reflashed at once). NVS was not erased; the existing Windows bond kept working.

Tool: `tests/hardware-probe` (`just hw-test COM3 COM7`), which uses the real protocol crate over the USB console (COM3) and Bluetooth SPP (COM7).

| Check | Result |
|---|---|
| Boot: key store opens before the radio; `Device 3c8a1f5d75ac: no key`; pairing window opens | ✅ |
| USB console answers `ClearKey` in the Bluetooth image | ✅ |
| Unprovisioned: `AuthBegin` → `Unprovisioned`; commands applied; one Medium warning | ✅ |
| `Rediscover` → v2 snapshot, 3 LED registrations, brightness replay | ✅ |
| `SetState 250` clamped to 100 (1B, **B-V6**) | ✅ |
| USB `ProvisionKey` → 256-bit key; repeated → same key | ✅ |
| `ProvisionKey` over Bluetooth refused; no key on the radio | ✅ |
| Keyed: unauthenticated command rejected (High SysLog); `Rediscover` still works | ✅ |
| Wrong key: the desktop rejects the device's proof and sends nothing | ✅ |
| Mutual handshake; signed command applied; replayed and tampered commands rejected; plain command rejected while authenticated | ✅ |
| Colour show (GPIO12, GPIO14, GPIO27, all, mix, off): every signed command confirmed | ✅ |
| Reconnect: rediscovery replays the real brightness (2A **A-V6**, Bluetooth part); the new connection is unauthenticated | ✅ |
| Key survives a reset; `ClearKey` survives a reset | ✅ |
| Device left unprovisioned at the end | ✅ |
| Pairing window (1C, **C-V6**, timing part): opens at 1.40 s, "pairing window closed; device is no longer discoverable" at 121.43 s, so 120.03 s | ✅ |

The first run failed one check (the Bluetooth port could not be reopened). The cause was in the probe: its reader thread kept the port handle open. Fixed in the probe, then 23/23.

Not covered on hardware: the Wi-Fi image; unplugging USB (A-V6 USB part); a second, unbonded device trying to pair (C-V6 rejection part); a telemetry flood; the Tauri GUI itself (the probe exercises the same protocol paths, not the desktop code).

## 6. Register changes

| Finding | Change |
|---|---|
| PIN-TPL-003 | In Progress → **Resolved** for paired devices. Residual by decision: an unpaired device is open and the UI warns |
| PIN-TPL-010 | Open → **Resolved** |
| PIN-TPL-013 | Open → **Resolved**. Residual: Wi-Fi traffic is unencrypted; the token and credentials are compiled in (PIN-TPL-015) |
| PIN-TPL-016 | In Progress → **Resolved** |

Totals: **7 Open · 8 In Progress · 12 Resolved** (27). The Critical watch-list is empty, on condition that actuators are only enabled on a paired device.

## 7. Recovery

Revert the 2B commits, or `git switch phase-2a/protocol-v1` and reflash. A key left in NVS is ignored by older firmware.

## 8. Next

Phase 2C, Firmware runtime and sensor modules (PIN-TPL-009, -019, -027, and the rest of -008).
