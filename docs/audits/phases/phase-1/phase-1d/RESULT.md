> **Correction (2026-11-05).** The LiDAR planner described here kept its own position and ignored the servos. That was a flaw in this phase, now fixed: see PIN-TPL-005 in [findings/high.md](../../../findings/high.md). The claim that scan positions stay inside the ROI held only for the plan, not for a clamped servo.

# Phase 1D — Actuator and LiDAR safety (RESULT)

| Field | Value |
|---|---|
| Phase | 1D |
| Status | **Completed — Verified** against the hardware-independent criteria D-V1…D-V4. The supervised hardware check D-V5 is **not done** (yours to run) |
| Approved | 2026-10-31. No option was chosen, so the recommended default applies: coils de-energize when idle, with a per-instance `set_hold_when_idle(true)` opt-in |
| Branch | `phase-1d/actuator-safety` (local, not pushed), cut from `phase-1c/trust-boundaries` @ `e85b2826` |
| Findings | PIN-TPL-004, PIN-TPL-005 → **Resolved** (Verified after D-V5) |
| Active image | **Unchanged.** Release ELF section sizes are identical to Phase 1C (text 1 117 970, data 251 566, bss 42 539), because the changed modules are not instantiated |
| Wire format | Unchanged (`protocol/` and `UI/` have no diff since 1C) |

## Commits

| Commit | Step | Content |
|---|---|---|
| `a407c118` | — | 1D plan |
| `5aec9215` | D1 | Pure `stepper_motion.rs` core; `stepper.rs` reduced to the timer check, coil writes and event emission |
| `4123676b` | D2 (+D3 for LiDAR) | Pure `lidar_scan.rs` planner; `lidar.rs` uses it, validates ROI and step, flushes every 100 points; `first_emit` no longer moves; explicit `run_self_test()` |
| `1e1e5a63` | D3 | `ServoModule::new` writes no PWM; explicit `run_self_test()`; `set_offset` error instead of `unwrap()` |
| `80bf102f` | D4 | `disabled_wiring.rs` hazard notes, `Module` lifecycle docs, README, module-boundaries, contract notes C-13/C-16 |

## Behavior now (all in modules not instantiated by the current image)

| Module | Before 1D | After 1D |
|---|---|---|
| Servo | `new` drove the servo to its offset, then swept 35°/10°/−10°/−35° (every LiDAR servo too). `set_offset` panicked on I²C errors | `new` writes no PWM; the servo is unpowered until its first command. The sweep is the explicit `run_self_test()`. Errors are returned |
| Stepper | Started in `Homing`: swept ±90° three times as soon as ticked | Starts `Idle`; homing only via `SetMode{Homing}` |
| Stepper | `MoveToAngle` and pivot limits unbounded | Finite and within ±180° travel; pivot `min ≤ max`. Otherwise rejected (High `SysLog` via the 1B dispatcher) and nothing moves |
| Stepper | Pivot/homing targets relative to the current position. **Reproduced:** from 180°, pivot-to-max gained one step per tick forever (22 048 half-steps after 20 000 ticks) | Absolute targets: settles exactly at the pivot limit |
| Stepper | `MoveToOrigin`/`MoveToPivot*` took at most one step unless already moving | Enter `Moving`/`Pivot` and complete the move |
| Stepper | Coils stayed energized when idle | Released on entering `Idle` (unless `set_hold_when_idle(true)`) |
| Stepper | GPIO errors in shift-register mode panicked; manual-mode errors were ignored | Returned (tick errors are rate-limited `SysLog`s, from 1B) |
| LiDAR | Termination needed exact equality. **Reproduced:** step 3 (span 140) and step 0 never ended (1 000 000 points and counting) | Every move is clamped to the limit; every scan ends after `cols × rows` points |
| LiDAR | Buffer flushed only at row ends | Flushed every 100 points; buffer ≤ 100 |
| LiDAR | `max_chunk` from `(|max.x|·2)·(|min.y|·2)/100`; ROI and step unvalidated; `first_emit` swept both servos | `max_chunk = ceil(cols·rows/100)` with checked arithmetic; ROI within ±90°, step 1–180, otherwise rejected; `first_emit` only reports the ROI; the sweep is the explicit `run_self_test()` |

The default LiDAR scan (step 1, ±90°) produces the same positions as before: 181 × 181 = 32 761 points, matching the old algorithm's count in simulation.

## Verification

| # | Check | Result |
|---|---|---|
| D-V1 | Host tests | ✅ **72 pass in debug and release** (was 53). `stepper_motion` 12: no steps from a new stepper over 50 000 ticks; move then idle with exactly one coil release; holding option; timer-not-due; travel rejection for 180.5, −181, 1e30, `f32::MAX`, NaN and ∞; pivot validation; **pivot from 180° settles at 90° without exceeding it**; homing three sweeps within ±90° then idle; origin completes; `SetMode Idle` stops in place and does not resume; coil errors propagate. `lidar_scan` 7: **3 240 ROI/step combinations** all terminate with point count = `cols × rows`, positions inside the ROI, buffer ≤ 100 and the expected chunk count; the finding's case; default scan size; serpentine order; step and ROI rejection; messages |
| D-V2 | Code review | ✅ No PWM, GPIO or servo calls in `ServoModule::new`, `StepperMotor::new`, `Lidar::new` or `Lidar::first_emit` (checked by extracting each body and grepping for hardware writes: 0 hits). `Lidar::new` only creates the (now motion-free) servos and starts the rangefinder, a sensor. No `.unwrap()` left in `modules/actuators` or `modules/composite` |
| D-V3 | Other suites and builds | ✅ Tauri 4 and frontend 22 tests pass; `tsc` clean; protocol golden passes, with no diff in `protocol/` or `UI/` since 1C. Firmware debug and release pass, with warnings **11 → 7** (LiDAR's unused imports and dead `step_y` removed). rustfmt clean on 4 crates. No new clippy warnings in the host crate |
| D-V4 | Mutation check | ✅ Stepper starts in `Homing` → 2 failures. Travel check removed → 2 failures. Old relative pivot target reintroduced → 1 failure. LiDAR steps not clamped → 2 failures. Tree restored clean |
| D-V5 | Supervised hardware check | ⏳ **Not run.** See below |

**Not covered by tests.** The coil and PWM writes, the step timer, and the LiDAR tick plumbing (range read → record → flush → move) use ESP-IDF types. They were verified by build and review only; they delegate every decision to the tested cores.

### Supervised hardware check (D-V5) for you to run

Only on a bench, with the mechanism free to move and power within reach:
1. Uncomment one servo or stepper block in `board/disabled_wiring.rs` and wire it into `board::install_active_modules`. That is a source change for the test only; do not commit it.
2. Flash and boot. **Expect no motion** at power-up or registration.
3. Send a valid command, e.g. `MoveToAngle{angle: 30}` or servo `SetAngle{angle: 20}`. Expect the motion, then stepper coils released when done.
4. Send `MoveToAngle{angle: 500}`. Expect no motion and a "Command failed: stepper angle 500 is outside the travel limit ±180°" `SysLog`.

## Effect on findings and the safety gate

| Finding | Before → After |
|---|---|
| PIN-TPL-004 | Open → **Resolved** (Verified after D-V5) |
| PIN-TPL-005 | Open → **Resolved** (Verified after D-V5) |

**Critical watch-list.** The entries for 004 and 005 are cleared. PIN-TPL-003 remains: there is no application-level authentication, so any device paired during the 120 s window could drive an enabled actuator.

**Safety gate.** The roadmap's literal criterion (1B, 1C and 1D Completed — Verified) is now met for the **hardware-independent** checks. **Recommendation: keep actuators out of the default image** until the hardware checks B-V6, C-V6 and D-V5 have passed, and preferably until PIN-TPL-003 is closed in 2B.

No new findings. No severity changes.

## Recovery

Revert the 1D commits, or `git switch phase-1c/trust-boundaries`.

## Next step

Stop for review. **Phase 1 is complete** (1A–1D). Next in the roadmap: **Phase 2A, Protocol contract v1** (PIN-TPL-007, -012, -017, -021). This is the first *versioned wire-format change*: shared fixtures so the TypeScript schemas are checked against Rust, a `protocol_version` field, a proper rediscovery command with current-state replay, child routing and USB-disconnect reporting. It needs your authorization and one decision: whether to correct the misspelled wire names now (with input aliases), or keep them for compatibility.
