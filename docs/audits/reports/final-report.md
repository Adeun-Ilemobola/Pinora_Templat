# Pinora Template — Final report (audit and modernization, 2026-10-31 … 2026-11-02)

## Outcome

| | Baseline (`main` @ `5b5c8ed4`) | Now (`phase-3a/cleanup`) |
|---|---|---|
| Findings | 27: 0 Critical, 6 High, 13 Medium, 8 Low, all Open | **34 Resolved, 2 Blocked** of 36 (both wire-breaking renames, waiting for a protocol v3). 9 were added on 2026-11-05 |
| Critical watch-list | 4 High findings that became Critical once actuators were enabled | Empty, on condition that actuators run only on a paired device |
| Tests | None | protocol 31, firmware host 146, Tauri 21, frontend 173, on-device `just hw-test` 23 + 5; mutation-checked at every phase |
| Warnings | 17 firmware, 3 clippy | 0 everywhere; CI denies clippy warnings |
| CI | None | `.github/workflows/ci.yml`, **green on GitHub** (run 41532290661, 2026-11-02) |
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

All work was done on **11 chained local branches**, each cut from the previous one. **On 2026-11-02, at your request, `main` was fast-forwarded to `phase-3a/cleanup` (`cf61f79b`, after the history fix below)**: 67 commits on top of the baseline `5b5c8ed4`, with no merge commit and no rewritten history. The phase branches are kept. **`main` was pushed to GitHub on 2026-11-02** (fast-forward from `5b5c8ed4`).

1. Nothing pending from this programme. CI is green on `main`.
2. Later work:
   - Protocol v3, which removes the 2 Blocked findings (`parent_id` `""` → `null`; output spellings). Do it together with the sibling projects.
   - Audits of `pinora_cli`, `pinora_studio`, `2dLidar` and `UI`. These are out of scope here; contract C-10 waits on them.

## After the push (2026-11-02)

The first CI runs found two problems that the Windows machine could not show. Both are fixed, and CI is now green on both jobs:

- **The host-test crate did not build on Linux.** Its `#[path]` includes went through directories that do not exist (`src/core/transport/..`). Windows normalizes such paths as text, while Linux resolves them through the filesystem. The crate now includes each firmware file as a flat, hidden module and mirrors the tree with re-exports.
- **CI's stable Rust (1.99) has newer clippy lints** than the local 1.97: `chunks_exact` should become `as_chunks`, and `fetch_update` is deprecated. Both are fixed in a way that still builds on the firmware's `esp-1.93`. The local stable toolchain was updated to 1.99. `actions/checkout` moved to v5 (Node 20 deprecation).

Before the first push, 6 399 Cargo build files committed by mistake in Phase 3A, including a generated copy of the local `wifi_config.rs`, were removed from the six unpushed commits. After the push, the local backups, `refs/original` and the affected reflogs were deleted and the objects pruned, so no copy of that file remains in the local repository either. Your pre-existing stash was left untouched. The build folder is now ignored.

## Floating-state audit (2026-11-05)

Requested by the maintainer after finding that the LiDAR planner ignored the servos. Nine more findings (PIN-TPL-028 to -036), all fixed. Two earlier claims were wrong and are corrected: "LiDAR scans always terminate" held only for the plan, and "stores for the latent module families" was left as later work although it meant a scan could not be seen at all.

Not verified: none of the changed modules has run on hardware (only LEDs are attached), the desktop window has not been opened with them, and `SERVO_MS_PER_DEGREE` is an assumption. Servos and the stepper have no position feedback, so "position" still means the last command that succeeded.
