# Phase 1D — Actuator and LiDAR safety (PLAN)

| Field | Value |
|---|---|
| Phase | 1D |
| Status | **Completed — Verified** (see [RESULT.md](RESULT.md)). Approved 2026-10-31 |
| Decision | The user authorized 1D without choosing, so the **recommended default** applies: stepper coils **de-energize when idle**, with a per-instance `set_hold_when_idle(true)` opt-in for holding torque |
| Branch | `phase-1d/actuator-safety`, cut from `phase-1c/trust-boundaries` @ `e85b2826` (local, not pushed) |
| Linked findings | PIN-TPL-004, PIN-TPL-005 |
| Scope note | All affected modules are **not instantiated** in the current image (3 LEDs). The active firmware's behavior is unchanged |
| Wire format | Unchanged (no protocol change; golden file byte-identical) |

## Approach: pure motion cores (the "output seam")

The roadmap asked for a seam that lets the actuator state machines run on the host. As in 1B (`led_mapping`, `dispatch`), the decision logic moves into **pure, hardware-free files**, which `tests/firmware-host` compiles and tests. The hardware modules keep only the I/O:

- `modules/actuators/stepper_motion.rs`: modes, targets, limits, homing/pivot sequencing, coil-release decisions. The stepper module supplies a `try_step(sequence_index)` closure that checks the step timer and writes the coils.
- `modules/composite/lidar_scan.rs`: ROI and step validation, the serpentine scan plan, termination, and point and chunk counts. The LiDAR module records ranges and moves the servos to the positions it returns.

This tests the same decisions as output traits would, with far less churn in ESP-IDF-typed code.

## Changes

| Step | Change |
|---|---|
| D1 | **Stepper.** Starts `Idle` (it used to start `Homing`, sweeping ±90° three times on the first ticks). Homing and pivot happen only on command. Targets are validated: `MoveToAngle` must be finite and within the travel limit (±180°); pivot limits must be finite, within travel, and keep `min ≤ max`. Violations are rejected with an error, which the 1B dispatcher reports as a High `SysLog`, and nothing moves. Pivot and homing use **absolute** targets (fixing the existing relative-target defect). `MoveToOrigin` and `MoveToPivotMax/Min` now enter a moving mode (fixing the existing single-step defect). Coils are released when the motor becomes `Idle`, unless holding is enabled. Coil-write errors propagate instead of panicking or being ignored. `SetMode` and the pivot moves report the applied mode with `GetMode` |
| D2 | **LiDAR scan.** ROI corners must lie within the servo pivot range (±90°). `SetStep` must be 1–180. Scan positions step toward each row or column limit **clamped to the limit**, and a row or scan ends when the limit is reached, so every scan terminates after `cols × rows` points. The point buffer is flushed whenever it reaches 100 points, so it stays bounded. `max_chunk` is computed from the actual scan size (`cols × rows`) with checked arithmetic, not the old formula |
| D3 | **No motion at boot.** `ServoModule::new` no longer writes the PWM; the servo stays unpowered until its first command, and the old sweep becomes the explicit `run_self_test()`. `Lidar::first_emit` reports the ROI without moving; the old corner sweep becomes the explicit `run_self_test()`. `set_offset`'s `.unwrap()` becomes error propagation |
| D4 | Docs: module docs, `board/disabled_wiring.rs` hazard notes, contract notes, architecture docs |

## Intended behavior changes (all in non-instantiated modules)

1. No servo, stepper or LiDAR motion during construction or registration.
2. Stepper: commands outside the limits are rejected and reported; no unbounded travel; coils off when idle; the origin and pivot commands now complete their moves.
3. LiDAR: scans always finish; memory is bounded; `max_chunk` is accurate.

## Verification

| # | Check | Pass criterion |
|---|---|---|
| D-V1 | Host tests: new `stepper_motion` and `lidar_scan` tests, plus the existing 53 | Pass in debug and release. These include: a stepper idle after `new` takes no steps; out-of-range targets are rejected; pivot from a far position reaches the absolute limit (the old code ran away); release on idle; every ROI/step combination terminates within the bound, with point count = `cols × rows` and a buffer ≤ 100 |
| D-V2 | Code review | No PWM/GPIO writes in `ServoModule::new`, `StepperMotor::new`, `Lidar::new` or `Lidar::first_emit` (grep evidence) |
| D-V3 | Golden tests, firmware builds, clippy, fmt, `bun test`, Tauri tests | Pass; golden unchanged |
| D-V4 | Mutation check | Reverting the clamped stepping, the travel check or the idle start makes tests fail |
| D-V5 | Optional supervised hardware check (yours) | Enable one servo or stepper on a bench: no motion at boot; out-of-range commands rejected |

## Recovery

Revert the 1D commits, or `git switch phase-1c/trust-boundaries`.
