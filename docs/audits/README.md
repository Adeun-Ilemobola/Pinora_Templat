# Pinora Template — Audit & Modernization Record

> **Authoritative copy.** This directory (`Pinora_Templat/docs/audits/`) is the source of truth. A synchronized mirror lives at `C:\Dev\Pinora\Audits\Pinora_Templat\`.

| | |
|---|---|
| **Project** | Pinora Template (`Pinora_Templat`, v0.8.0-alpha) |
| **Scope** | `C:\Dev\Pinora\Pinora_Templat` only. `2dLidar`, `pinora_cli`, `pinora_studio` and `UI` are **not audited** |
| **Audit date** | 2026-10-31 |
| **Baseline** | `main` @ `5b5c8ed4`, clean working tree |
| **Current operation** | **Roadmap complete** (Phases 0, 1A–1D, 2A–2E, 3A) |
| **Current stage** | Phase 3A **Completed — Verified** ([result](phases/phase-3/phase-3a/RESULT.md)); [final report](reports/final-report.md). Roadmap approved ([roadmap](phases/ROADMAP.md)) |
| **Approval status** | ▶ You authorized the remaining phases (2C, 2D, 2E, 3A) without stopping between them, and on-device testing (2026-11-02). Next: **your review and merge decision**; then protocol v3 (017, 021) and the sibling-project audits |
| **Source changes so far** | Phase 0 (structural), 1A (tests, scripts, CI), 1B (command safety), 1C (pairing window, COM validation, UI error containment), 1D (actuator and LiDAR safety), 2A (protocol v1, rediscovery, frontend contract), 2B (framing, drop accounting, Wi-Fi robustness, device authentication and USB pairing), 2C (1 ms scheduler, ordered registry, IR and RFID fixes), 2D (bounded logs, async commands, store slices), 2E (fresh-clone builds, pinned patches, pin map), 3A (zero warnings, CSP, UART0 ownership, docs) on chained local branches ending at `phase-3a/cleanup`. Not pushed; `main` unchanged at `5b5c8ed4` |
| **Last synchronization** | 2026-11-02: mirror verified by SHA-256 over all files (see the mirror's `SYNC-STATUS.md`) |

## Severity totals

| Critical | High | Medium | Low | Total |
|:-:|:-:|:-:|:-:|:-:|
| **0** | **6** | **13** | **8** | **27** |

0 Open · 0 In Progress · 2 Blocked · **25 Resolved** (022, 023, 024, 026 in Phase 3A; 017 and 021 Blocked on a major protocol version; 015, 018 in Phase 2E; 011, 014, 020 in Phase 2D; 008, 009, 019, 027 in Phase 2C; 001, 002, 025 in Phase 1B; 006 in Phase 1C; 004, 005 in Phase 1D; 007, 012 in Phase 2A; 003, 010, 013, 016 in Phase 2B). Until Phase 2A this line said 13 Open; the register had 12. The reassessment re-rated PIN-TPL-016 Medium → Low and added PIN-TPL-027 (Low). The [Critical escalation watch-list](findings/critical.md) is empty since Phase 2B, on condition that actuators are enabled only on a paired device.

## Most important risks

1. ~~**Firmware stops obeying after one failed command**~~ (PIN-TPL-001). **Resolved in Phase 1B**: failures are reported and the loop continues.
2. ~~**A device paired over Bluetooth can send commands without further authentication**~~ (PIN-TPL-003). **Resolved in Phase 2B** for paired devices: pair over USB, and every command is then authenticated and signed. An unpaired device stays open, with a warning.
3. ~~**An out-of-range LED value reboots debug builds or mis-drives the LED**~~ (PIN-TPL-002). **Resolved in Phase 1B**: values are clamped to 100 with a warning.
4. ~~**Dormant motor code moves hardware on boot and accepts unlimited travel; the LiDAR scan may never finish**~~ (PIN-TPL-004, -005). **Resolved in Phase 1D**: no motion at construction, bounded stepper targets, scans always terminate.
5. ~~**One malformed line from the board can crash the desktop UI**~~ (PIN-TPL-006). **Resolved in Phase 1C**: Tauri validates COM lines, the log helpers never throw, and route errors are contained.
6. ~~**A hand-copied protocol** makes changes risky~~ (PIN-TPL-007). **Resolved in Phase 2A**: protocol v1 carries a version, and the TypeScript schemas are tested against the Rust golden fixture. Rediscovery replays current state on every transport (PIN-TPL-012).

## Issue summary

| ID | Sev | Title | Status |
|---|---|---|---|
| PIN-TPL-001 | High | One failed command stops the firmware main loop | Resolved |
| PIN-TPL-002 | High | LED brightness unchecked; overflow corrupts output or reboots | Resolved |
| PIN-TPL-003 | High | Bluetooth accepts any nearby device; no app authorization | Resolved |
| PIN-TPL-004 | High | Actuators move uncommanded; unbounded targets *(latent)* | Resolved |
| PIN-TPL-005 | High | LiDAR scan never terminates; unbounded memory *(latent)* | Resolved |
| PIN-TPL-006 | High | Malformed device input crashes the desktop log view | Resolved |
| PIN-TPL-007 | Med | Rust↔TS contract handwritten, partial, unversioned | Resolved |
| PIN-TPL-008 | Med | No automated tests or CI | Resolved |
| PIN-TPL-009 | Med | Main loop busy-polls; no timing guarantees | Resolved |
| PIN-TPL-010 | Med | Telemetry backpressure and silent drops | Resolved |
| PIN-TPL-011 | Med | Frontend log history unbounded, copied per message | Resolved |
| PIN-TPL-012 | Med | Connection-state desync; no state replay | Resolved |
| PIN-TPL-013 | Med | Wi-Fi dev-grade auth; fragile boot dependency | Resolved |
| PIN-TPL-014 | Med | Tauri commands block the UI thread | Resolved |
| PIN-TPL-015 | Med | Fresh clone does not build; machine-specific config | Resolved |
| PIN-TPL-016 | Low | Duplicated framing (dead code removed; re-rated from Medium) | Resolved |
| PIN-TPL-017 | Med | Module identity and hierarchy incomplete | Blocked |
| PIN-TPL-018 | Med | Hard-coded pins; LED on strapping pin GPIO12 | Resolved |
| PIN-TPL-019 | Med | Defects in inactive RFID, IR and rangefinder modules | Resolved |
| PIN-TPL-020 | Med | Frontend god-store, oversized files, `any` | Resolved |
| PIN-TPL-021 | Low | Misspellings (wire names frozen, internal fixable) | Blocked |
| PIN-TPL-022 | Low | Formatting and lint not enforced | Resolved |
| PIN-TPL-023 | Low | Tauri scaffold metadata, CSP off, leftovers | Resolved |
| PIN-TPL-024 | Low | Documentation drift | Resolved |
| PIN-TPL-025 | Low | Events report requested, not applied values | Resolved |
| PIN-TPL-026 | Low | UART0 driver bypasses HAL ownership | Resolved |
| PIN-TPL-027 | Low | Module tick/registration order changes every boot | Resolved |

Full records: [findings/](findings/README.md).

## Outstanding problems and next step

- **Done:** Phase 0, Structural Reorganization ([plan](phases/phase-0/PLAN.md), [result](phases/phase-0/RESULT.md)). Decisions: D1–D3 yes, D4 no, D5–D6 yes. One local commit per step (P0.1–P0.7) on `phase-0/structural-reorganization`; all checks V1–V10 pass; no contract, manifest, lockfile or config changed.
- **Your review:** inspect the branch, optionally run the hardware smoke test in RESULT §8, and decide whether to merge it into `main`. Merging and pushing are your decisions.
- **Done:** post-Phase-0 reassessment ([report](reports/post-phase-0-reassessment.md)). All findings re-verified; no new problems introduced by Phase 0; one new pre-existing finding (027); 016 re-rated Low.
- **Done:** roadmap approved; **Phase 1A** completed ([result](phases/phase-1/phase-1a/RESULT.md)): 28 host tests for hardware-free firmware code, `just test`, CI workflow (not yet run on GitHub).
- **Done:** **Phase 1B** ([result](phases/phase-1/phase-1b/RESULT.md)): failing commands no longer stop the firmware; tick errors reported; LED brightness clamped; overflow-safe math; applied-value events. 46 host tests; mutation-checked.
- **Done:** **Phase 1C** ([result](phases/phase-1/phase-1c/RESULT.md)): 120 s Bluetooth pairing window; Tauri validates COM lines (non-protocol lines → `espConsole`); log helpers total; route error boundary. Defaults applied: 120 s, `espConsole`.
- **Done:** **Phase 1D** ([result](phases/phase-1/phase-1d/RESULT.md)): pure stepper and LiDAR cores; no motion at construction; bounded targets; terminating scans; coils released when idle (default applied). **Phase 1 complete.**
- **Safety gate:** met for hardware-independent checks. Keep actuators out of the default image until hardware checks B-V6, C-V6 and D-V5 pass and, preferably, until 2B closes PIN-TPL-003.
- **Done:** **Phase 2A** ([result](phases/phase-2/phase-2a/RESULT.md)): protocol v1 (version field, `Rediscover`, input aliases); firmware rediscovery with state replay and child routing; USB disconnect event; complete Zod schemas enforced by the golden fixture; validation on receive and send. Default applied: misspelled output names kept. 79 host, 5 Tauri, 137 frontend tests.
- **Done:** **Phase 2B** ([result](phases/phase-2/phase-2b/RESULT.md)): one framer and limit table; counted drops and rate limits; Wi-Fi boot without an AP; listener deadline and replacement; protocol v2 mutual HMAC authentication, signed commands, USB-only pairing. Verified on your ESP32 over USB and Bluetooth (23/23 + 5/5).
- **Done:** **Phase 2C** ([result](phases/phase-2/phase-2c/RESULT.md)): 1 ms fixed-rate loop (measured mean 1.000 ms, p99 ≤ 1.2 ms on your board), construction-ordered registry, RMT IR capture, RFID record format and bounded buffer.
- **Done:** **Phase 2D** ([result](phases/phase-2/phase-2d/RESULT.md)): bounded, batched log history with export; slow Tauri commands off the main thread; store split into slices with strict typing.
- **Done:** **Phase 2E** ([result](phases/phase-2/phase-2e/RESULT.md)): fresh clones build without `wifi_config.rs`; ESP-IDF patches pinned; pin map with checks. **Phase 2 complete.**
- **Done:** **Phase 3A** ([result](phases/phase-3/phase-3a/RESULT.md)): zero warnings enforced in CI, CSP, UART0 ownership, documentation fixed. **Roadmap complete.**
- **Next:** your review; merging into `main` and pushing are your decisions ([final report](reports/final-report.md)).
- **Validation gaps:** hardware checks done on 2026-11-02 for the Bluetooth image (B-V6 clamp, A-V6 reconnect replay, B2-V7). Still not run: Wi-Fi image, USB unplug, a second unbonded device, actuators (D-V5; none attached). Stepper and LiDAR decision logic is host-tested since 1D; sensor modules (2C) and the ESP-IDF I/O paths are verified by build and review only. CI has not executed on GitHub.

## Documents

| Area | Document |
|---|---|
| Architecture as implemented | [architecture/current-system.md](architecture/current-system.md) |
| Module boundaries | [architecture/module-boundaries.md](architecture/module-boundaries.md) |
| Communication flow | [architecture/communication-flow.md](architecture/communication-flow.md) |
| Frozen contracts | [architecture/compatibility-contracts.md](architecture/compatibility-contracts.md) |
| Issue register | [findings/README.md](findings/README.md) · [critical](findings/critical.md) · [high](findings/high.md) · [medium](findings/medium.md) · [low](findings/low.md) |
| Phases | [phases/README.md](phases/README.md) · [Roadmap](phases/ROADMAP.md) · [Phase 0 plan](phases/phase-0/PLAN.md) · [Phase 0 result](phases/phase-0/RESULT.md) · [Phase 1A plan](phases/phase-1/phase-1a/PLAN.md) · [Phase 1A result](phases/phase-1/phase-1a/RESULT.md) · [Phase 1B plan](phases/phase-1/phase-1b/PLAN.md) · [Phase 1B result](phases/phase-1/phase-1b/RESULT.md) · [Phase 1C plan](phases/phase-1/phase-1c/PLAN.md) · [Phase 1C result](phases/phase-1/phase-1c/RESULT.md) · [Phase 1D plan](phases/phase-1/phase-1d/PLAN.md) · [Phase 1D result](phases/phase-1/phase-1d/RESULT.md) · [Phase 2A plan](phases/phase-2/phase-2a/PLAN.md) · [Phase 2A result](phases/phase-2/phase-2a/RESULT.md) · [Phase 2B plan](phases/phase-2/phase-2b/PLAN.md) · [Phase 2B result](phases/phase-2/phase-2b/RESULT.md) · [Phase 2C plan](phases/phase-2/phase-2c/PLAN.md) · [Phase 2C result](phases/phase-2/phase-2c/RESULT.md) · [Phase 2D plan](phases/phase-2/phase-2d/PLAN.md) · [Phase 2D result](phases/phase-2/phase-2d/RESULT.md) · [Phase 2E](phases/phase-2/phase-2e/RESULT.md) · [Phase 3A](phases/phase-3/phase-3a/RESULT.md) |
| Report | [reports/initial-audit.md](reports/initial-audit.md) · [reports/post-phase-0-reassessment.md](reports/post-phase-0-reassessment.md) · [reports/final-report.md](reports/final-report.md) |
| Evidence | [evidence/validation-baseline.md](evidence/validation-baseline.md) · [evidence/probes.md](evidence/probes.md) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |

## Version control

- `docs/audits/` is committed on the local branches (`phase-0/structural-reorganization`, then `phase-1a/host-test-harness`, then `phase-1b/command-safety`, then `phase-1c/trust-boundaries`, then `phase-1d/actuator-safety`, then `phase-2a/protocol-v1`, then `phase-2b/transport-auth`, then `phase-2c/runtime-sensors`, then `phase-2d/desktop`, then `phase-2e/reproducible-builds`, then `phase-3a/cleanup`, which contains everything). It is not on `main` until you merge.
- The workspace mirror `C:\Dev\Pinora\Audits\` is **not under version control**: `C:\Dev\Pinora` is not a Git repository, and no repository was created.
- Pre-existing `docs/architecture-audit.md` (an earlier review) was left unchanged.

## Conventions

- Issue IDs `PIN-TPL-NNN` are permanent. Findings are never deleted; status moves Open → Planned → In Progress → Resolved → Verified (or Blocked).
- Phase statuses: Proposed · Awaiting Approval · Approved · In Progress · Blocked · Completed — Unverified · Completed — Verified.
- Source references are repository-relative paths with line numbers at the stated baseline commit, so these documents stay valid when copied elsewhere.
