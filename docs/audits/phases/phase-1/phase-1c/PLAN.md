# Phase 1C — Trust boundaries (PLAN)

| Field | Value |
|---|---|
| Phase | 1C |
| Status | **Approved** 2026-10-31 |
| Decisions | The user authorized 1C without choosing between the options, so the **recommended defaults** apply: a **120 s** pairing window, and a **new `espConsole` event** for non-protocol COM lines. Both are single constants or one event name, easy to change |
| Branch | `phase-1c/trust-boundaries`, cut from `phase-1b/command-safety` @ `3578e795` (local, not pushed) |
| Linked findings | PIN-TPL-003 (pairing-policy part; application-level authentication stays for 2B), PIN-TPL-006 |
| Wire format | Unchanged (protocol crate untouched; golden file byte-identical). **IPC contract C-07 gains one additive event**, `espConsole` |

## Changes

| Step | Change | Files |
|---|---|---|
| C1 | **Pairing window.** A new pure `core/transport/pairing_window.rs`. After the SPP server starts, the device is discoverable and accepts SSP pairing for 120 s. After that it becomes connectable but **not discoverable** and **rejects new pairings**. Already-bonded peers (bonds are kept in NVS) still connect. The window reopens only on reboot. The SSP confirmation uses the safe `EspGap::reply_ssp_confirm` instead of the raw `unsafe` call | `core/transport/bluetooth.rs`, new `core/transport/pairing_window.rs`, `core/transport/mod.rs` |
| C2 | **Validate COM lines in Tauri.** A new `line_classifier.rs`: a line that parses as a Rust `ProtocolMessage` goes out as `espState` (unchanged text); anything else goes out as the new `espConsole` event. This is the same validation the Wi-Fi path already applies | `UI/src-tauri/src/line_classifier.rs` (new, with unit tests), `serial_transport.rs`, `lib.rs` |
| C3 | **Total log helpers.** `describeLog`/`matchesLog` never throw; unrecognized shapes become an "Unrecognized message" entry | `UI/src/lib/logs.ts`, `UI/tests/logs.test.ts` (the PIN-TPL-006 expectations flip deliberately) |
| C3 | **Route error boundary.** A rendering fault shows an error panel *inside* the layout, so the sidebar and transport controls stay usable, with "Clear log history" and "Open dashboard" actions | new `UI/src/components/RouteError.tsx`, `UI/src/App.tsx` |
| C3 | **Console listener.** `espConsole` lines go to `console.debug("ESP:", …)`, exactly where non-JSON lines went before | `UI/src/lib/Modulefront.ts` |
| C4 | Docs and CI: the Bluetooth guide (pairing window), README, contract note C-07, architecture docs; CI runs the Tauri unit tests | as listed, `.github/workflows/ci.yml` |

## Intended behavior changes

1. New devices can pair only during the first 120 s after the Bluetooth service starts. Afterwards the board is not discoverable and rejects pairing. The bonded desktop keeps connecting.
2. Lines on USB serial or Bluetooth that are not valid protocol messages no longer reach the frontend's protocol handling or log history; they arrive as `espConsole`. This includes firmware console output and malformed or unknown JSON.
3. The log view can no longer crash on an unexpected message shape. If any route does fail to render, the error is contained to that route.

## Verification

| # | Check | Pass criterion |
|---|---|---|
| C-V1 | Host tests (`tests/firmware-host`) | New `pairing_window` tests pass in debug and release; the existing 46 still pass |
| C-V2 | Tauri unit tests (`cargo test` in `UI/src-tauri`) | Classifier tests pass |
| C-V3 | `bun test`, `tsc --noEmit`, Vite build | Pass, with the updated expectations |
| C-V4 | Protocol golden tests; firmware `esp-1.93` debug and release builds; Tauri clippy; rustfmt | Pass; golden unchanged; no new warnings |
| C-V5 | Mutation check | Reverting the window check, the classifier or the log fallback makes tests fail |
| C-V6 | Optional hardware check (yours) | Pair within 120 s; a second device cannot discover or pair after 120 s; the bonded desktop reconnects; console lines no longer appear in the Logs page |

## Recovery

Revert the 1C commits, or `git switch phase-1b/command-safety`. Bonds made during testing can be removed in Windows Bluetooth settings.
