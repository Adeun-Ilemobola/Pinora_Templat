# Phase 1B — Firmware command safety (PLAN)

| Field | Value |
|---|---|
| Phase | 1B |
| Status | **Approved** 2026-10-31. Decision: out-of-range LED values are **clamped** to 0–100, with a warning |
| Branch | `phase-1b/command-safety`, cut from `phase-1a/host-test-harness` @ `ee600d69` (local, not pushed) |
| Linked findings | PIN-TPL-001, PIN-TPL-002, PIN-TPL-025 |
| Behavioral impact | **Yes, intended** (listed below). No wire-format change: only existing message shapes are used, and the golden file must stay byte-identical |

## Changes

| Step | Change | Files |
|---|---|---|
| B1 | **Overflow-safe range math.** `range_u32`, `range_i32` and `range_i16` compute in `i128` and saturate to the output type. `pulse_us_to_tick` computes in `i64`. Results are identical wherever the old code did not overflow | `Firmware_Templates/src/utilities/math.rs` |
| B2 | **LED clamp.** `SetState` values above 100 are clamped to 100. A `SysLog` (Medium) reports the clamp. `Brightness` reports the *applied* level, and the stored state is the applied level. The pure mapping lives in a new `led_mapping.rs` so it is host-testable | `modules/actuators/led.rs`, new `modules/actuators/led_mapping.rs`, `modules/actuators/mod.rs` |
| B3 | **Non-fatal command dispatch.** A new pure `runtime/dispatch.rs`. A handler `Err` is logged on the console and reported as a `SysLog` (High) from the failing module's id, and the loop continues. Unknown ids and the reserved `"SSI"` keep their current behavior | `runtime/dispatch.rs` (new), `runtime/mod.rs` |
| B3 | **Tick errors reported, rate-limited.** A new pure `runtime/tick_errors.rs`. The first tick error per module is reported immediately as a `SysLog` (Medium); after that, at most one report per module every 5 s, carrying the count of suppressed errors. `now_us()` is read only when an error occurs | `runtime/tick_errors.rs` (new), `runtime/mod.rs` |
| B4 | **Applied values (PIN-TPL-025).** Servo `GetMinPivot`/`GetMaxPivot` report the stored (clamped) limits; stepper `GetPivotMin`/`GetPivotMax` report the stored values | `modules/actuators/servo.rs`, `modules/actuators/stepper.rs` |
| B5 | **Docs.** Module docs, `runtime` error-semantics docs, root README firmware section, contract notes C-13/C-16 | as listed |

Out of scope (roadmap 1D): the `.unwrap()` calls and uncommanded motion in servo, stepper and LiDAR. The active image (3 LEDs) has no `.unwrap()` in its command path.

## Intended behavior changes

1. A failing command no longer ends `main`. The firmware keeps processing later commands, and the desktop receives a `SysLog` naming the module.
2. Tick errors are no longer silent. They arrive as rate-limited `SysLog`s.
3. `SetState { state > 100 }` drives 100 % and reports `level: 100`, plus a clamp warning. Previously it drove full or wrapped duty and reported the raw value, and debug builds panicked above 524 352.
4. Servo pivot-limit events report the clamped values.

## Verification

| # | Check | Pass criterion |
|---|---|---|
| B-V1 | Host tests | Updated characterization tests (overflow → saturation) and new tests for `led_mapping`, `dispatch` and `tick_errors` pass in debug and release |
| B-V2 | Protocol golden tests | Byte-identical (no wire change) |
| B-V3 | Firmware `esp-1.93` debug and release builds | Pass; warnings ≤ 11; ELF size change reported |
| B-V4 | `just test`, clippy, fmt | Pass; no new warnings in changed files |
| B-V5 | Mutation check | Reverting the clamp or the non-fatal dispatch makes tests fail |
| B-V6 | Optional hardware check (yours) | `SetState{state: 600000}` → LED at 100 %, `Brightness{level: 100}` plus a clamp `SysLog`, no reboot |

## Recovery

Revert the 1B commits, or `git switch phase-1a/host-test-harness`.
