# Phase 1B — Firmware command safety (RESULT)

| Field | Value |
|---|---|
| Phase | 1B |
| Status | **Completed — Verified** against the hardware-independent criteria B-V1…B-V5. The hardware check B-V6 is **not done** (yours to run) |
| Approved | 2026-10-31. Decision: clamp out-of-range LED values |
| Branch | `phase-1b/command-safety` (local, not pushed), cut from `phase-1a/host-test-harness` @ `ee600d69` |
| Findings | PIN-TPL-001, PIN-TPL-002, PIN-TPL-025 → **Resolved** (Verified after B-V6) |
| Wire format | Unchanged: the protocol crate and golden file have no diff |

## Commits

| Commit | Step | Content |
|---|---|---|
| `8c8f9504` | — | 1B plan |
| `bc181a67` | B1 | Overflow-safe `range_u32`/`range_i32`/`range_i16` (`i128` + saturate) and `pulse_us_to_tick` (`i64`) |
| `ce51ed59` | B2 | LED clamp to 0–100, a Medium `SysLog` on clamp, `Brightness` reports the applied level; new pure `modules/actuators/led_mapping.rs` |
| `c0f58fbe` | B3 | New pure `runtime/dispatch.rs` (non-fatal handler errors → High `SysLog`) and `runtime/tick_errors.rs` (rate-limited Medium `SysLog`, 5 s per module); `runtime::run` now returns `!` |
| `939e532b` | B4 | Servo and stepper pivot-limit events report the stored values |
| `9551123c` | B5 | Docs: module trait, root README, `docs/architecture-audit.md`, audit architecture docs, contract notes C-13/C-16 |

## Behavior now (intentional changes)

| Situation | Before 1B | After 1B |
|---|---|---|
| A command handler returns `Err` | `?` ended `main`. The main task exited, LEDs went to idle and every later command was ignored, silently | Console error, plus `SysLog{priority: High, text: "Command failed: <error>", raw_err}` from the module's id. The loop continues |
| `tick()` returns `Err` | Discarded | The first error per module is sent immediately as `SysLog{Medium, "Tick failed: <error>"}`. Afterwards at most one per module every 5 s: `"… (<n> more since last report)"` |
| `SetState{state: 101…u32::MAX}` | Duty full or wrapped, reported `level: state`. Debug builds panicked (reboot) above 524 352 | Duty 100 %. `SysLog{Medium, "LED brightness <n> is out of range 0-100; applied 100"}`, then `Brightness{level: 100}`. Stored state is 100, so `Toggle` then goes to 0 |
| `range_*` with overflowing inputs | Panic (debug) or wrap (release) | Saturate at the type limits. Identical results wherever the old math did not overflow |
| Servo `SetMin/MaxPivot` beyond the other limit | Reported the requested value | Reports the clamped, stored value |

Unknown module ids (console log only) and the reserved `"SSI"` id (ignored) behave as before.

## Verification

| # | Check | Result |
|---|---|---|
| B-V1 | Host tests | ✅ **46 pass in debug and in release** (was 28). New: `led_mapping` 6, `dispatch` 5, `tick_errors` 4; `math` grew from 10 to 13, including grid **equivalence tests** proving new == old wherever the old formula did not overflow (>100 `u32` and >50 `i32` cases compared) |
| B-V2 | Protocol golden tests | ✅ 2/2; `protocol/` has no diff since 1A |
| B-V3 | Firmware `esp-1.93` debug and release builds | ✅ pass, 11 warnings (unchanged, all pre-existing). Release ELF text 1 113 390 → **1 117 198 (+3 808 B)**, data 250 958 → **251 310 (+352 B)**, bss 42 539 (=). This is expected: `i128` arithmetic helpers, new message strings, the tick-error map |
| B-V4 | `just test`, fmt, clippy | ✅ protocol 2 + host 46 + frontend 16 pass; rustfmt clean on 4 crates; no new clippy warnings in changed or new files |
| B-V5 | Mutation check (each fix reverted in turn, then restored) | ✅ LED clamp removed → 2 failures. Command-failure `SysLog` removed → 1 failure. Rate limit disabled → 3 failures. Old overflowing `range_u32` → 2 failures. Tree restored clean |
| B-V6 | Hardware check | ⏳ **Not run.** See below |

**Compiler-enforced guarantee.** `runtime::run` now has return type `!`, so the main loop cannot exit with an error. The old fail-stop path is impossible to reintroduce by accident.

**What is not covered by tests.**
- The loop body in `runtime/mod.rs` and `LedModule::set_state` both use ESP-IDF types. They were verified by build and review: they only call the tested pure functions.
- The servo and stepper event changes (B4) are verified by build and diff review only. Those modules are not host-testable until the 1D output-trait seam exists.

### Hardware check (B-V6) for you to run

1. Flash: `cd Firmware_Templates; cargo +esp-1.93 run --frozen --release`. This writes to the board.
2. Start `bun run tauri dev` and connect over Bluetooth.
3. Open the webview devtools (right-click → Inspect) and send an out-of-range value, using the UUID shown on an LED card ("ID · …"): `window.__TAURI_INTERNALS__.invoke("send_data", {data: JSON.stringify({id: "<led uuid>", command: {Led: {SetState: {state: 600000}}}})})`.
4. Expected: the LED is at full brightness and the board does not reboot. Logs show a clamp `SysLog` and `Brightness level 100`. A following `Toggle` turns it off.

## Effect on findings

| Finding | Before → After | Evidence |
|---|---|---|
| PIN-TPL-001 | Open → **Resolved** | `runtime/dispatch.rs`, `runtime/tick_errors.rs`, `run() -> !`; tests in `dispatch.rs`/`tick_errors.rs` |
| PIN-TPL-002 | Open → **Resolved** | `led_mapping.rs`, `math.rs`; tests in `led_mapping.rs`/`math.rs` |
| PIN-TPL-025 | Open → **Resolved** | `servo.rs`, `stepper.rs` (build and review) |

Each moves to **Verified** after B-V6. The Critical watch-list entry for PIN-TPL-001 is cleared. 004, 005 and 003 remain, and the safety gate still needs 1C and 1D.

No new findings. Observation: the clamp `SysLog` is sent before the duty is applied, so if `set_duty` fails, the desktop sees the warning followed by a "Command failed" report. That is acceptable and documented here.

## Recovery

Revert the 1B commits, or `git switch phase-1a/host-test-harness`. `main` is unchanged at `5b5c8ed4`.

## Next step

Stop for review. Next in the roadmap: **Phase 1C, Trust boundaries** (PIN-TPL-003 pairing policy, PIN-TPL-006 desktop input validation). It needs your authorization and two decisions: the Bluetooth pairing-window length (120 s suggested), and whether to add an `espConsole` event for non-protocol lines (recommended) or drop them.
