# Phase 1C — Trust boundaries (RESULT)

| Field | Value |
|---|---|
| Phase | 1C |
| Status | **Completed — Verified** against the hardware-independent criteria C-V1…C-V5. The hardware check C-V6 is **not done** (yours to run) |
| Approved | 2026-10-31. No option was chosen, so the recommended defaults apply: 120 s pairing window, new `espConsole` event |
| Branch | `phase-1c/trust-boundaries` (local, not pushed), cut from `phase-1b/command-safety` @ `3578e795` |
| Findings | PIN-TPL-006 → **Resolved**. PIN-TPL-003 → **In Progress** (pairing policy done; application-level authentication remains for 2B) |
| Wire format | Unchanged (`protocol/` has no diff since 1B). IPC contract C-07 gains the additive event `espConsole` |

## Commits

| Commit | Step | Content |
|---|---|---|
| `6ca78db2` | — | 1C plan |
| `143f8dac` | C1 | Pure `core/transport/pairing_window.rs` (120 s). The Bluetooth worker opens the window when SPP starts and, on expiry, switches to connectable/non-discoverable. The GAP callback accepts SSP confirmations only while the window is open |
| `cd1906a6` | C2 | Tauri `line_classifier.rs`: COM lines that parse as a Rust `ProtocolMessage` → `espState` (text unchanged); others → `espConsole`. Unit tests |
| `9c503b41` | C3 | `describeLog`/`matchesLog` are total ("Unrecognized message" fallback). `RouteError` per-route error boundary plus a root fallback. `espConsole` listener → `console.debug`. Updated characterization tests |
| `f8f11b6c` | C4 | Bluetooth guide, README, UI README, architecture review, audit architecture docs, contract notes C-04/C-07. CI runs the Tauri unit tests. New `just test-tauri` recipe |
| `8afd9a02` | C4 | Event-flow diagram fix |

## Behavior now (intentional changes)

| Situation | Before 1C | After 1C |
|---|---|---|
| New Bluetooth device tries to pair | Accepted at any time, and the device was discoverable forever | Accepted only within **120 s** of SPP start. Afterwards the board is not discoverable, new pairing confirmations are answered *no*, and the console logs `Bluetooth pairing rejected: the pairing window is closed`. Bonded devices still connect. Power-cycle to reopen |
| Firmware console text on USB serial / SPP | Forwarded as `espState`; the frontend skipped non-`{` lines with `console.debug` | Forwarded as `espConsole` → `console.debug("ESP:", …)`. It never reaches protocol handling or the log history |
| JSON on COM that is not a protocol message (`{}`, `{"Foo":1}`, a `ModuleEvent` with an empty event, a truncated line) | Reached the store, entered the log history, and **crashed the Logs view** | Sent as `espConsole`; never reaches the store |
| An unrecognized shape somehow reaches the log helpers | `TypeError` during render | An "Unrecognized message" entry |
| Any route throws during render | React Router's default error screen replaced the whole app | An error panel inside the layout, with sidebar and transport controls still usable, plus "Clear log history" and "Open dashboard" |

The Wi-Fi path is unchanged: it already validated lines and dropped invalid ones.

## Verification

| # | Check | Result |
|---|---|---|
| C-V1 | Host tests | ✅ **53 pass in debug and release** (7 new `pairing_window` tests: closed before start, accepted only within the window, expiry reported once, opens once per boot, no panic on clock skew) |
| C-V2 | Tauri unit tests | ✅ 4 pass: valid lines forwarded unchanged (including `\r\n` and padding); console text; `{}`, unknown, empty-event, wrong-payload, array and `null` JSON; truncated and invalid UTF-8 |
| C-V3 | `bun test`, `tsc`, Vite build | ✅ 22 pass (was 16; nine fallback cases replace the three "throws" expectations); `tsc` clean; bundle 646.52 kB |
| C-V4 | Golden tests, firmware builds, clippy, fmt | ✅ `protocol/` has no diff since 1B; golden passes. Firmware debug and release pass, 11 warnings (unchanged); release ELF text 1 117 198 → **1 117 970 (+772 B)**, data 251 310 → **251 566 (+256 B)**, bss unchanged. Tauri clippy 2 warnings (pre-existing); rustfmt clean on 4 crates |
| C-V5 | Mutation check (each fix reverted, then restored) | ✅ Window never closes → 1 failure. Classifier accepts everything → 3 failures. `ModuleEvent` shape guard removed → 7 failures. Tree clean |
| C-V6 | Hardware check | ⏳ **Not run.** See below |

### Notes

- **SSP confirmation path.** `esp-idf-svc`'s safe `reply_ssp_confirm` is gated on `cfg(esp_idf_bt_ssp_enabled)`. ESP-IDF 5.5 no longer has the `CONFIG_BT_SSP_ENABLED` option (SSP is built in), so the wrapper is compiled out. The firmware therefore keeps the direct `esp_bt_gap_ssp_confirm_reply` call, the same one that was already reported working, and now passes the window decision instead of a constant `true`. Changing `sdkconfig` to re-enable the wrapper was out of scope (contract C-17).
- **Not covered by tests.** The GAP callback and the worker's window open/close calls use ESP-IDF types. They were verified by build and review only; they call the tested `PairingWindow` and an atomic flag. Whether Windows treats a declined Just Works confirmation as a failed pairing (the expected behavior) is part of C-V6. There is no component test for `RouteError`: it would need a DOM test library, which means new dev dependencies and a lockfile change.
- **What the window does not do.** Any device paired during the 120 s can still send commands. Application-level authentication is Phase 2B, so PIN-TPL-003 stays In Progress, and its Critical watch-list entry stays until 2B.

### Hardware check (C-V6) for you to run

1. Flash: `cd Firmware_Templates; cargo +esp-1.93 run --frozen --release`. This writes to the board. If the desktop is already bonded, it should reconnect without pairing.
2. **Within 120 s of boot:** pair a device in Windows Bluetooth settings. Expect success.
3. **After the console shows `Bluetooth pairing window closed`:** try to discover or pair from a *second* device. Expect it to not see the board, or the pairing to fail. The console shows `Bluetooth pairing rejected`.
4. Reconnect the bonded desktop (`bun run tauri dev` → Bluetooth → Connect). Expect the LED cards to appear.
5. Over USB serial, check that ESP-IDF log lines no longer appear on the Logs page; they show in the webview devtools console as `ESP: …`.

## Effect on findings

| Finding | Before → After | Evidence |
|---|---|---|
| PIN-TPL-006 | Open → **Resolved** | `line_classifier.rs` (+ tests), `logs.ts` (+ tests), `RouteError.tsx`, `App.tsx` |
| PIN-TPL-003 | Open → **In Progress** | `pairing_window.rs` (+ tests), `bluetooth.rs`. Remaining: application-level authentication (2B) |
| PIN-TPL-012 | Open (unchanged) | Note: USB serial read errors are still not reported to the UI (2A) |

No new findings, and no severity changes. The safety gate for enabling actuators still needs 1D, and app-level authentication (2B) to clear PIN-TPL-003 from the watch-list.

## Recovery

Revert the 1C commits, or `git switch phase-1b/command-safety`. Remove any test bonds in Windows Bluetooth settings.

## Next step

Stop for review. Next in the roadmap: **Phase 1D, Actuator and LiDAR safety** (PIN-TPL-004, -005): no motion at construction, bounded targets, terminating scans, bounded memory, and a host-testable output seam. It needs your authorization and one decision: whether stepper coils de-energize when idle by default (recommended) or keep a per-instance holding option.
