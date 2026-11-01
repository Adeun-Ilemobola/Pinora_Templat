# Phase 2A — Protocol contract v1 (PLAN)

| Field | Value |
|---|---|
| Phase | 2A |
| Status | **Completed — Verified** 2026-11-01 (approved 2026-10-31); see [RESULT.md](RESULT.md) |
| Decision | The user authorized 2A without choosing on the misspelled wire names, and no recommendation had been given. The **conservative default** applies: **keep** `lool_up_id`, `Ckick`, `Idol` and `distant` on output (C-01 and C-10 consumers are unaudited), and **accept** the corrected spellings on input as serde aliases. The output rename is deferred to a future major protocol version |
| Branch | `phase-2a/protocol-v1`, cut from `phase-1d/actuator-safety` @ `daa3022e` (local, not pushed) |
| Linked findings | PIN-TPL-007, PIN-TPL-012, PIN-TPL-017, PIN-TPL-021 |
| Wire format | **Versioned, additive change (protocol v1).** Existing messages keep their shape; one field and one command variant are added. The golden file changes, and every changed line is reviewed |

## Changes

| Step | Change | Files |
|---|---|---|
| P1 | **Protocol v1.** `PROTOCOL_VERSION = 1`. `SystemInfo.protocol_version` (`#[serde(default)]`, so pre-v1 firmware parses as 0). New `ModuleCommand::System(SystemCommand::Rediscover {})`, addressed to the reserved id `"SSI"`. Input aliases `lookup_id`, `Click`, `Idle` and `distance`. Golden file re-blessed (diff reviewed); new tests for aliases and pre-v1 compatibility | `protocol/src/**`, `protocol/tests/**` |
| P2 | **Firmware rediscovery and state replay.** `dispatch` recognizes `SSI` + `Rediscover` and the loop then re-sends a **fresh** system snapshot, every registration and every module's `first_emit`. `first_emit` now replays current state: LED `Brightness`, stepper mode and pivot limits. **Child routing:** `Module::child_mut` lets a composite expose its children; dispatch falls back to children when no top-level id matches (LiDAR exposes its servos and rangefinder). Duplicate lookup names are reported at registration. System snapshot carries `protocol_version` | `runtime/**`, `core/module.rs`, `core/hardware/system_info.rs`, `modules/actuators/{led,stepper}.rs`, `modules/composite/lidar.rs`, host tests |
| P3 | **USB serial disconnect.** The COM reader emits a new additive `serialConnection: "Disconnected"` event on a USB read error (Bluetooth already had one) | `UI/src-tauri/src/serial_transport.rs` |
| P4 | **Frontend contract.** Complete Zod `ProtocolMessageSchema` / `IncomingCommandSchema` covering every Rust family, plus a `bun test` that parses **every golden fixture** with them (Rust↔TS drift now fails tests). Incoming messages are validated with Zod; outgoing commands are typed and validated. Registry and logs are cleared *before* connecting. A rediscovery request is sent after every connect. USB loss is shown. LED state is "unknown until reported". Protocol version shown, with a mismatch warning. Duplicate lookup names produce a warning | `UI/src/lib/protocol/*`, `UI/src/lib/{Modulefront,IncomingCommand}.ts`, `UI/src/lib/Modules/Led/*`, `UI/src/components/Esp32StatsCard.tsx`, `UI/tests/*` |
| P5 | Docs: contract notes (C-01 v1, C-05, C-06, C-07), communication-flow, README | as listed |

Deferred, with reasons: `parent_id: "" → null` and the output rename of misspellings (both wire-breaking, so a future major version); TypeScript code generation (the fixture test enforces the same guarantee without new dependencies).

## Compatibility

| Pairing | Result |
|---|---|
| New desktop + old firmware | Works. `protocol_version` reads as 0 and the UI shows "pre-v1" with a warning. Old firmware logs the `Rediscover` command as invalid and ignores it, and Serial still has no replay |
| Old desktop + new firmware | Works. Rust ignores unknown fields, the old Zod `SystemInfoSchema` is non-strict, and the old desktop never sends `System` commands |

## Verification

| # | Check | Pass criterion |
|---|---|---|
| A-V1 | Protocol tests | Golden diff limited to the intended lines (new `protocol_version` field, new command case). Alias and pre-v1 tests pass |
| A-V2 | Host firmware tests | New dispatch (rediscover, child routing), duplicate-name and replay tests pass; existing tests pass |
| A-V3 | Frontend | Fixture test parses all golden lines; `tsc`; Vite build; `bun test` |
| A-V4 | Builds and lint | Firmware debug and release, Tauri test and clippy, rustfmt |
| A-V5 | Mutation check | Removing a family from the TS schema, the rediscover branch, or the child fallback makes tests fail |
| A-V6 | Optional hardware check (yours) | Reconnect over Bluetooth or Serial: LED cards show the real brightness; unplugging USB shows the lost-connection error |

## Recovery

Revert the 2A commits, or `git switch phase-1d/actuator-safety`.
