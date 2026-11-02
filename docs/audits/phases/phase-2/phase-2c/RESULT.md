# Phase 2C — Firmware runtime and sensor modules (RESULT)

| Field | Value |
|---|---|
| Phase | 2C |
| Status | **Completed — Verified**: host checks pass; loop timing, module order and the protocol suite were verified on your ESP32. The IR and RFID changes are host-tested only (no receiver or reader attached) |
| Date | 2026-11-02 |
| Branch | `phase-2c/runtime-sensors`, cut from `phase-2b/transport-auth` @ `3bf9e4ce` (local, not pushed) |
| Decisions | Taken as defaults under your blanket approval: 1 kHz FreeRTOS tick and a blocking 1 ms loop; RFID "Pinora record v1"; RMT IR capture (see the plan) |
| Findings | PIN-TPL-008, -009, -019, -027 → **Resolved** |
| Wire format | Unchanged (golden file untouched) |

## 1. Commits

| Step | Commit | Change |
|---|---|---|
| Plan | `8d058974` | Phase 2C plan |
| C1 | `8d6c00f4` | Pure NEC decoder; RMT capture for the IR receiver; RFID record format, bounded write buffer, reader-version check |
| C2 | `2ce4dd2c` | Insertion-ordered `ModuleRegistry` |
| C3 | `49929faa` | 1 ms scheduler, `LoopStats`, `CONFIG_FREERTOS_HZ=1000`, wait compensation |
| C4 | *(docs commit)* | Register, contracts, RESULT |

## 2. On-device measurements (ESP32 rev v3.1, LED image, Bluetooth)

Each line is one 10 s window, as logged by the firmware.

| Version | Iterations | Period mean | p50 | p99 | max | Work mean / max | Overruns |
|---|---|---|---|---|---|---|---|
| Before 2C | busy loop, 1 ms yield every 650 ms (not instrumented) | — | — | — | — | — | — |
| First 2C build: block for the full remaining time | 5 501 | 1 818 µs | ≤ 2 100 | ≤ 2 200 | 2 664 | 5 / 203 µs | 0 |
| | 5 465 | 1 830 µs | ≤ 2 100 | ≤ 2 200 | 8 444 | 5 / 156 µs | 0 |
| **Final: one tick less, then sleep to the tick boundary** | 9 999 | **1 000 µs** | ≤ 1 100 | **≤ 1 200** | 2 041 | 5 / 119 µs | 0 |
| | 9 985 | 1 001 µs | ≤ 1 100 | ≤ 1 200 | 11 263 | 5 / 48 µs | 0 |
| | 9 988 | 1 001 µs | ≤ 1 100 | ≤ 1 200 | 11 612 | 5 / 685 µs | 0 |

**Why the first build was slow.** ESP-IDF's `pthread_cond_timedwait`, which Rust's `recv_timeout` uses, rounds a timeout up to whole ticks and then adds one tick. A wait of just under 1 ms therefore lasts 1–2 ms. The final loop asks for one tick less, and when under one tick remains it sleeps to the next tick boundary (`vTaskDelay(1)`). A command arriving during that sleep waits at most one tick.

**Against the plan's criterion (C-V3):** mean period 1 ms ± 0.1 ms ✅; p99 ≤ 2 ms ✅ (≤ 1.2 ms); no task-watchdog warnings ✅; the CPU is free about 99% of the time (5 µs work per 1 ms). Once per 10 s window, one iteration takes 11–12 ms. The likely cause is a higher-priority task (Bluetooth, logging); it was not traced further.

| Other on-device check | Result |
|---|---|
| Registration order on three boots | `led1, led2, led3` every time (construction order) ✅ |
| `just hw-test` (protocol v2 scenario, key persistence) | 23/23 and 5/5 ✅ |

## 3. Verification

| # | Check | Result |
|---|---|---|
| C-V1 | Host tests | ✅ **118 pass in debug and release** (was 97): NEC decoder (all 256 command bytes, timing spread, corruption, truncation, repeat code), card record (round trips up to 747 bytes, legacy UUID cards, rendering, truncated, unknown version and corrupt-length records, bounded buffer, reader versions), registry order over 20 runs with random ids, replacement keeps position, schedule (grid, overrun skip, wait planning, simulated 10 s loop blocks over 80% of the time), histogram percentiles, overrun counting |
| C-V2 | Builds | ✅ Firmware debug and release with `--frozen`; warnings 6 → 5. Release image 1 032 567 / 233 158 / 31 467 (text / data / bss), +5 932 text against 2B; it fits the 1 536 000-byte app partition. Debug image 1 662 340 / 520 817 / 45 636. rustfmt clean |
| C-V3 | On device | ✅ §2 |
| C-V4 | Mutation | ✅ Registry iterating its hash index instead of the ordered list → 2 failures. Wait planning without the tick compensation → 1 failure. Restored |

## 4. Not verified

- The IR receiver on RMT, and the RFID format on a real card: no receiver or MFRC522 is attached. Both compile against the pinned esp-idf-hal (the new RMT driver API), and their logic is host-tested.
- The 11–12 ms worst-case iteration was observed, not explained.
- The stepper and LiDAR timing under the new loop: these modules are latent, with no actuators attached.

## 5. Register changes

| Finding | Change |
|---|---|
| PIN-TPL-008 | In Progress → **Resolved** (test suites, `just test`, CI workflow, `just hw-test`; CI has not run on GitHub because nothing is pushed) |
| PIN-TPL-009 | Open → **Resolved** (verified on your board) |
| PIN-TPL-019 | Open → **Resolved** (host-tested) |
| PIN-TPL-027 | Open → **Resolved** (verified on your board) |

Totals: **4 Open · 7 In Progress · 16 Resolved** (27).

## 6. Recovery

Revert the 2C commits (including `sdkconfig.defaults`), or `git switch phase-2b/transport-auth` and reflash. Cards written in record v1 are not readable by older firmware, which expected one 16-byte UUID.

## 7. Next

Phase 2D, Desktop performance and structure (PIN-TPL-011, -014, -020).
