# Pinora Template — Final report (audit and modernization, 2026-10-31 … 2026-11-02)

## Outcome

| | Baseline (`main` @ `5b5c8ed4`) | Now (`phase-3a/cleanup`) |
|---|---|---|
| Findings | 27: 0 Critical, 6 High, 13 Medium, 8 Low, all Open | **25 Resolved, 2 Blocked** (both wire-breaking renames, waiting for a protocol v3) |
| Critical watch-list | 4 High findings that became Critical once actuators were enabled | Empty, on condition that actuators run only on a paired device |
| Tests | None | protocol 31, firmware host 122, Tauri 21, frontend 167, on-device `just hw-test` 23 + 5; mutation-checked at every phase |
| Warnings | 17 firmware, 3 clippy | 0 everywhere; CI denies clippy warnings |
| CI | None | `.github/workflows/ci.yml` (not run on GitHub yet: nothing is pushed) |
| Fresh clone | Every Rust build failed without a hand-made `wifi_config.rs` | Builds and passes all host suites as is |

## What changed, by area

- **Firmware runtime.** A failed command no longer stops the loop (1B). The busy loop is now a 1 ms fixed-rate scheduler: mean period 1.000 ms, p99 ≤ 1.2 ms on your board (2C). Module order equals construction order on every boot (2C). Telemetry drops are counted and reported (2B).
- **Safety.** LED values are clamped. Actuators do not move at construction and have bounded targets. LiDAR scans always terminate (1B, 1D).
- **Protocol.** v1 added a version field, rediscovery with current-state replay, and child routing (2A). v2 added authentication (2B). The TypeScript schemas are enforced against the Rust golden file (2A).
- **Security.** The Bluetooth pairing window is limited to 120 s (1C). Pairing over USB (2B) gives mutual HMAC-SHA256 authentication with signed, replay-protected commands; the keys never reach the WebView. A paired device is read-only to unauthenticated peers. A Content Security Policy is set (3A).
- **Desktop.** COM lines are validated (1C). History is bounded and batched with export. Slow commands run off the main thread. The store is split into slices with strict typing (2D).
- **Builds.** ESP-IDF patches are pinned and unused `rand` removed (2E); `embassy-time` turned out to be needed. There is a board pin map with checks. UART0 ownership is enforced (3A).

## Your board (ESP32 rev v3.1, MAC 3c:8a:1f:5d:75:ac, LEDs on GPIO 12/14/27)

It currently runs the **Phase 3A image**, flashed with the project's bootloader and partition table, and it is **unprovisioned** (no device key). To require authentication, connect over USB in the app and click **Pair this device**. `just hw-test COM3 COM7` reruns the on-device checks; it leaves the device unprovisioned.

## Not verified

- **Wi-Fi image on hardware.** It needs the Wi-Fi image and your local network credentials.
- **IR receiver and RFID on hardware.** None is attached.
- **Actuators.** None is attached.
- **The desktop app's window.** Not launched; the store tests mock the Tauri IPC, and the CSP was not checked in the running app.
- **Bluetooth edge cases.** Rejection of a second, unbonded device, and USB unplug.
- **The CI workflow on GitHub.**

## Decisions taken as defaults under your approval (2026-11-02)

| Decision | Choice | Where |
|---|---|---|
| Scheduler | 1 kHz FreeRTOS tick and a blocking 1 ms loop | [2C](../phases/phase-2/phase-2c/PLAN.md) |
| RFID card format | "Pinora record v1"; legacy UUID cards still read | 2C |
| Log history | 5 000 entries; Export to JSON | [2D](../phases/phase-2/phase-2d/PLAN.md) |
| GPIO12 | Kept (your wiring) | [2E](../phases/phase-2/phase-2e/RESULT.md) |
| Firmware target directory | `C:/t` kept, with a `CARGO_TARGET_DIR` override | 2E |
| App identifier | `com.owner.ui` kept (moving it moves the paired-device keys) | [3A](../phases/phase-3/phase-3a/RESULT.md) |

Pairing over USB, read-only enforcement and RustCrypto were your explicit choices for 2B.

## Git state and your next steps

All work was done on **11 chained local branches**, each cut from the previous one. **On 2026-11-02, at your request, `main` was fast-forwarded to `phase-3a/cleanup` (`aa80dffc`)**: 67 commits on top of the baseline `5b5c8ed4`, with no merge commit and no rewritten history. The phase branches are kept. Nothing was pushed.

1. Push `main` when you want CI to run on GitHub (your decision; not done).
2. Later work:
   - Protocol v3, which removes the 2 Blocked findings (`parent_id` `""` → `null`; output spellings). Do it together with the sibling projects.
   - Audits of `pinora_cli`, `pinora_studio`, `2dLidar` and `UI`. These are out of scope here; contract C-10 waits on them.
   - Desktop stores for the latent module families.
