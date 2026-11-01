# Phase 2A — Protocol contract v1 (RESULT)

| Field | Value |
|---|---|
| Phase | 2A |
| Status | **Completed — Verified** against the hardware-independent criteria A-V1…A-V5. The optional hardware check A-V6 is **not done** (yours to run) |
| Date | 2026-11-01 |
| Branch | `phase-2a/protocol-v1`, cut from `phase-1d/actuator-safety` @ `daa3022e` (local, not pushed; `main` unchanged at `5b5c8ed4`) |
| Decision applied | No choice was made on the misspelled wire names, so the conservative default from the plan applies: output spellings kept, corrected spellings accepted as input aliases |
| Findings | PIN-TPL-007, PIN-TPL-012 → **Resolved**. PIN-TPL-017 Open → **In Progress**. PIN-TPL-021 stays **In Progress**. PIN-TPL-020 stays In Progress (outgoing commands typed) |
| Wire format | **Protocol v1, additive.** One field (`SystemInfo.protocol_version`) and one command variant (`System(Rediscover {})`) added; no existing line changed shape |
| Active image | **Changed** (rediscovery is live code): release text +10 116 B, data +416 B, bss unchanged |

## 1. Commits

| Step | Commit | Change |
|---|---|---|
| Plan | `04a15e19` | Phase 2A plan |
| P1 | `c6b31f38` | `PROTOCOL_VERSION = 1`; `SystemInfo.protocol_version` (`serde(default)`); `ModuleCommand::System(SystemCommand::Rediscover {})` on `SYSTEM_COMMAND_ID = "SSI"`; input aliases `lookup_id`, `Click`, `Idle`, `distance`; golden re-blessed; firmware snapshot reports the version |
| P2 | `03d43559` | Firmware: dispatch returns `Rediscover` to the main loop, which re-sends a fresh snapshot, every registration and every `first_emit`; LED and stepper `first_emit` replay current state; `Module::child_mut` and child fallback in dispatch (LiDAR exposes `servo_x`, `servo_y`, `rangefinder`); duplicate lookup names reported at registration |
| P3 | `de71ad3d` | Tauri: `serialConnection: "Disconnected"` on a USB read error (`TransportType::connection_event`); not sent after a deliberate shutdown |
| P4 | `fd551fe1`, `d7b5046d` | Frontend: Zod schemas for every family plus `System`; golden-fixture test; Zod validation on receive and send; state cleared before connecting; `Rediscover` after every connect; USB loss shown; LED "unknown until reported"; protocol version and mismatch warning; duplicate lookup-name warning. `d7b5046d` restores LF endings that the P4 edit script had converted to CRLF (content unchanged) |
| P5 | *(the docs commit after `d7b5046d`)* | Contract notes C-01, C-05, C-06, C-07; communication flow, current-system and module-boundaries notes; register, indexes, roadmap outcome, changelog; project `README.md`, `docs/architecture-audit.md`, Bluetooth and Wi-Fi guides; discovery-cache doc comment (comment only) |

## 2. Golden wire diff (reviewed)

Exactly two lines changed in `protocol/tests/golden/wire_format.txt`:

| Line | Change |
|---|---|
| `cmd.system.rediscover` | **New**: `{"id":"SSI","command":{"System":{"Rediscover":{}}}}` |
| `msg.system` | `"protocol_version":1` added as the first field |

Every other line is byte-identical, including the misspelled names (`lool_up_id`, `Ckick`, `Idol`, `distant`). New protocol tests: v1 constants; a pre-v1 snapshot (no field) parses as version 0; each alias is accepted on input and never emitted; `Rediscover` parses.

## 3. Verification

| # | Check | Result |
|---|---|---|
| A-V1 | Protocol tests | ✅ **6 pass**. Golden diff limited to the two intended lines (§2). Alias and pre-v1 tests pass |
| A-V2 | Host firmware tests | ✅ **79 pass in debug and release** (was 72). New: `Rediscover` on `"SSI"` returns to the loop and emits nothing itself; other commands on `"SSI"` stay `Reserved`; a `System` command sent to a module id is ignored by it; commands to a child id reach the child; a failing child command is reported from the **child's** id; unknown ids; `duplicate_lookup_names`; no children by default |
| A-V3 | Frontend | ✅ `bun test` **137 pass** (was 22): **every one of the 109 golden lines** parses with the TS schemas (32 commands with `IncomingCommandSchema`, 77 messages with `ProtocolMessageSchema`); family coverage checked (7 command, 10 event families); TS `PROTOCOL_VERSION` equals the fixture's; the `Rediscover` command the UI sends equals the fixture; pre-v1 snapshot → version 0; unknown families and out-of-range values rejected. `tsc --noEmit` clean. Vite build to scratch passes (the >500 kB chunk warning is pre-existing) |
| A-V4 | Builds and lint | ✅ Firmware debug and release build, warnings **7 → 6** (LED now uses its `EmitterError` import). Tauri **5 tests pass** (was 4); clippy shows only the 2 baseline warnings. Host-crate clippy: 3 `redundant_field_names` warnings, all on lines unchanged since 1D (`core/module.rs:80`, `emitter.rs:75-76`; PIN-TPL-022). rustfmt clean on 4 crates |
| A-V5 | Mutation check | ✅ Child fallback removed → 2 host failures. Rediscover branch removed → 1 host failure. `Rfid` event family removed from the TS schema → 5 fixture failures. `System` command family removed → 1 fixture failure. Every file restored and re-verified |
| A-V6 | Optional hardware check | ⏳ **Not run.** See §5 |

ELF sizes (`xtensa-esp32-elf-size`, text / data / bss):

| Image | 1D | 2A | Δ |
|---|---|---|---|
| Release | 1 117 970 / 251 566 / 42 539 | 1 128 086 / 251 982 / 42 539 | +10 116 / +416 / 0 |
| Debug | — | 1 613 532 / 504 593 / 45 636 | — |

## 4. Compatibility

| Pairing | Result | Evidence |
|---|---|---|
| New desktop + old (pre-v1) firmware | Works. The version reads as 0 and the statistics card warns "protocol v0; this app expects v1". Old firmware cannot parse `Rediscover` and only logs an error; Serial still has no replay with such firmware | Pre-v1 tests in Rust and TS |
| Old desktop + new firmware | Works. The old desktop never sends `System` commands, does not Zod-validate on receive, and its `SystemInfoSchema` is non-strict | Code review of the 1D frontend |
| Corrected spellings sent by a new client | Accepted on input; output unchanged | Alias tests |

## 5. Optional hardware check (A-V6), for you to run

No hardware was touched in this phase. If you choose to check on a board running the default image (LEDs only):

1. Flash the `phase-2a/protocol-v1` build yourself.
2. Connect over Bluetooth, set an LED to 40 %, disconnect and reconnect. **Expect** the LED card to show 40 % (not 0 % / Off), and the statistics card to say "protocol v1".
3. Connect over USB serial, unplug the cable. **Expect** "USB serial connection was lost" and the module list cleared.
4. Reconnect over USB without resetting the board, if your adapter allows it. **Expect** the modules to appear (rediscovery).

## 6. What is not done, and why

| Item | Finding | Reason |
|---|---|---|
| `parent_id: ""` → `null` | 017 | Wire-breaking; needs a future major protocol version |
| Emitter-level `"log"` pseudo-id reported as "Module store not found" in the console | 017 | Cosmetic; documented as reserved (C-05); left for 2D (frontend store work) |
| Output rename of `lool_up_id`, `Ckick`, `Idol`, `distant` | 021 | Wire-breaking for unaudited consumers (C-01, C-10); input aliases only |
| TypeScript code generation (`ts-rs` / `specta`) | 007 | Needs new dependencies; the fixture test enforces the same Rust↔TS agreement without them |
| Stores and dispatch for families other than Led and Button | 020 | Schemas now cover them; stores belong to 2D |

**Deviation from the roadmap's completion criterion.** The roadmap lists 007, 012, 017 **and** 021 as Resolved after 2A. The approved plan deferred the wire-breaking parts of 017 and 021 (the conservative default), so those two stay **In Progress**. Everything else in the plan is done.

## 7. New observations

- **Child routing widens the command surface of composites.** Commands can now address the LiDAR's servos directly, and a direct servo move does not update the scan position until the next scan or `ChangeMotorAngle` (documented in `lidar.rs`). The LiDAR is still not instantiated. This is noted on the Critical watch-list next to PIN-TPL-003.
- **Rediscovery right after a USB connect can be lost** if the DTR/RTS pulse resets the board, because the command arrives before the firmware reads input. The boot-time registrations cover that case, so the result is the same.

## 8. Register changes

| Finding | Change |
|---|---|
| PIN-TPL-007 | Open → **Resolved** (fixture-enforced schemas on both paths; protocol v1 version field; code generation deferred) |
| PIN-TPL-012 | Open → **Resolved** (Verified after A-V6) |
| PIN-TPL-017 | Open → **In Progress** (child routing, duplicate lookup names, reserved ids documented; `""`→`null` deferred) |
| PIN-TPL-021 | In Progress (input aliases added; output rename deferred) |
| PIN-TPL-020 | In Progress (outgoing commands typed and validated; rest in 2D) |

Totals: **9 Open · 10 In Progress · 8 Resolved** (27). The previous README said 13 Open; the register actually had 12, which is now corrected.

## 9. Recovery

Revert the 2A commits, or `git switch phase-1d/actuator-safety`. The golden file then returns to the unversioned format.

## 10. Next

**Phase 2B, Transport robustness and authentication** (PIN-TPL-010, -013, -016 and the authentication part of -003). It needs your decision on the crypto dependency and the device-key provisioning for the challenge-response handshake.
