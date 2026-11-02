# Phase 2C — Firmware runtime and sensor modules (PLAN)

| Field | Value |
|---|---|
| Phase | 2C |
| Status | **Approved** 2026-11-02 (you authorized all remaining phases and on-device testing) |
| Decisions (taken as defaults under that approval, recorded here) | **Scheduling:** a 1 ms fixed-rate loop that blocks on the command channel between ticks; this needs `CONFIG_FREERTOS_HZ=1000` (was 100: a blocking loop at 100 Hz would slow the stepper about 8×). **RFID card format:** "Pinora record v1", length-prefixed (magic `PN`, version, 16-bit length, payload); headerless cards still read as the legacy 16-byte UUID. **IR:** RMT hardware capture, polled from `tick()` |
| Branch | `phase-2c/runtime-sensors`, cut from `phase-2b/transport-auth` @ `3bf9e4ce` (local, not pushed) |
| Linked findings | PIN-TPL-009, PIN-TPL-019, PIN-TPL-027 (and the remaining part of PIN-TPL-008) |
| Wire format | Unchanged |

## Changes

| Step | Change | Findings |
|---|---|---|
| C1 | **Sensors.** Pure NEC decoder + RMT capture for the IR receiver (no pin sampling, no `unwrap`). RFID: record format, bounded write buffer, reader-version check restored. (Rangefinder error rate limit landed in 2B.) | 019, 009 |
| C2 | **Ordered registry.** `ModuleRegistry` keeps construction order (a `Vec` plus an id index); tick, registration and replay follow construction order on every boot | 027 |
| C3 | **Scheduler.** `CONFIG_FREERTOS_HZ=1000`. Each 1 ms period: tick every module, handle every pending command, then block on the command channel until the next period (commands wake the loop at once). `LoopStats` (pure) records period, tick-work time and overruns; a summary is logged every 10 s on the console | 009 |
| C4 | Docs, on-device measurement, RESULT | — |

## Verification

| # | Check | Pass criterion |
|---|---|---|
| C-V1 | Host tests | NEC decoder, card format and write buffer, registry order (repeated runs), loop statistics |
| C-V2 | Builds | Firmware debug and release; image fits the 1.5 MB app partition |
| C-V3 | On device | Measured over ≥ 10 s with the LED image: p99 period ≤ 2 ms, mean period 1 ms ± 0.1 ms; the idle task runs (no task-watchdog warnings); registration order equals construction order on two boots; `just hw-test` still passes |
| C-V4 | Mutation | Iterating the index instead of the ordered list, or not blocking between ticks, makes a test fail |

## Recovery

Revert the 2C commits (including `sdkconfig.defaults`), or `git switch phase-2b/transport-auth` and reflash.
