# Issue register — Pinora Template

Stable IDs `PIN-TPL-NNN`. IDs are never reused or renumbered. Findings are never deleted: when one is fixed, its status changes and its record stays.

Each finding record contains: severity, affected files with line references, what is wrong, why it matters, the effect on Pinora, evidence, the recommended solution, the risk of changing it, the verification method, confidence, status and proposed phase. Complex findings also have a plain-English explanation.

**Confidence:** *Confirmed* means it was proven by code or an executed probe. *Probable* means strong static evidence with one inferred step, which is named in the record. *Unverified* means plausible but not demonstrated.
**Status:** Open · Planned · In Progress · Blocked · Resolved · Verified.
**Roadmap phase** comes from the post-Phase-0 roadmap ([phases/ROADMAP.md](../phases/ROADMAP.md)), which is **pending approval**.

Baseline: commit `5b5c8ed4` (branch `main`), audited 2026-10-31. Line references in the records point at that commit. After Phase 0 the files moved: see the path map in [phases/phase-0/RESULT.md](../phases/phase-0/RESULT.md) §2.

## Totals

| Severity | Count | File |
|---|---|---|
| Critical | 0 | [critical.md](critical.md) (includes an escalation watch-list) |
| High | 6 | [high.md](high.md) |
| Medium | 13 | [medium.md](medium.md) |
| Low | 8 | [low.md](low.md) |
| **Total** | **27** | |

## Register

| ID | Sev. | Title | Area | Active image? | Confidence | Status | Roadmap phase |
|---|---|---|---|---|---|---|---|
| PIN-TPL-001 | High | A single failed command stops the firmware main loop | Firmware runtime | Yes | Confirmed / Probable (aftermath) | Resolved | 1B |
| PIN-TPL-002 | High | LED brightness not range-checked; overflow corrupts output or reboots | Firmware module | Yes | Confirmed (probe) | Resolved | 1B |
| PIN-TPL-003 | High | Bluetooth accepts any nearby device; no application authorization | Security / transport | Yes | Confirmed (static) | Resolved | 1C (+2B) |
| PIN-TPL-004 | High | Actuators move uncommanded at construction; unbounded targets | Firmware modules (hardware safety) | No (latent) | Confirmed (static) | Resolved | 1D |
| PIN-TPL-005 | High | LiDAR scan never terminates; unbounded memory growth | Firmware module | No (latent) | Confirmed (reasoning) | Resolved | 1D |
| PIN-TPL-006 | High | Malformed device input crashes the desktop log view | Desktop receive boundary | Yes | Confirmed (probe) / Probable (UI) | Resolved | 1C |
| PIN-TPL-007 | Medium | Rust↔TS contract handwritten, partial, unversioned, inconsistently validated | Communication | Yes | Confirmed | Resolved | 2A |
| PIN-TPL-008 | Medium | Firmware module logic not yet host-testable (harness + CI added in 1A) | Testing | Yes | Confirmed | In Progress | 1A |
| PIN-TPL-009 | Medium | Main loop busy-polls; no timing guarantees | Firmware runtime | Yes | Probable | Open | 2C |
| PIN-TPL-010 | Medium | Telemetry backpressure: stacked queues, silent drops, no rate limits | Communication | Partly | Confirmed | Resolved | 2B |
| PIN-TPL-011 | Medium | Frontend log history unbounded and copied per message | Desktop performance | Yes | Confirmed (static) | Open | 2D |
| PIN-TPL-012 | Medium | Connection state desync; timing-dependent discovery; no state replay | Communication / UI | Yes | Confirmed | Resolved | 2A |
| PIN-TPL-013 | Medium | Wi-Fi: dev-grade auth, single-thread listener, fatal boot dependency | Security / transport | No (selectable) | Confirmed / Probable | Resolved | 2B |
| PIN-TPL-014 | Medium | Tauri commands block the UI-facing thread | Desktop backend | Yes | Probable | Open | 2D |
| PIN-TPL-015 | Medium | Builds machine-specific; fresh clone does not build | Build | Yes | Confirmed / Probable | In Progress | 2E |
| PIN-TPL-016 | Low (was Medium) | Duplicated framing with divergent limits (dead code removed in Phase 0) | Structure | Yes | Confirmed | Resolved | 2B |
| PIN-TPL-017 | Medium | Module identity and hierarchy partially implemented | Architecture | Partly | Confirmed | In Progress | 2A |
| PIN-TPL-018 | Medium | Hard-coded pins; LED on strapping pin GPIO12; latent conflicts | Hardware config | Yes | Probable | In Progress | 2E |
| PIN-TPL-019 | Medium | Defects in inactive RFID, IR receiver and rangefinder modules | Firmware modules | No (latent) | Confirmed (static) | Open | 2C |
| PIN-TPL-020 | Medium | Frontend god-store and `any` typing (file split done in Phase 0) | Desktop structure | Yes | Confirmed | In Progress | 2D |
| PIN-TPL-021 | Low | Misspelled wire names (internal names fixed in Phase 0) | Naming | Yes | Confirmed | In Progress | 2A |
| PIN-TPL-022 | Low | Lint not enforced (formatting fixed in Phase 0) | Tooling | Yes | Confirmed | In Progress | 3A |
| PIN-TPL-023 | Low | Tauri scaffold metadata, CSP disabled, `greet`, lost error detail | Desktop packaging | Yes | Confirmed | Open | 3A |
| PIN-TPL-024 | Low | Documentation and legacy metadata drift | Documentation | Yes | Confirmed | In Progress | 3A |
| PIN-TPL-025 | Low | Events report requested rather than applied values | Firmware modules | No (latent) | Confirmed | Resolved | 1B |
| PIN-TPL-026 | Low | UART0 driver installed bypassing HAL ownership | Firmware | Yes | Confirmed | Open | 3A |
| PIN-TPL-027 | Low | Module tick/registration order changes every boot (HashMap) | Firmware runtime | Yes | Probable | Open | 2C |

"Active image?" means the defect is reachable in the firmware and desktop as currently built (LEDs over Bluetooth). "Latent" code is compiled but not instantiated by `main.rs`.

## Dependencies between findings

- PIN-TPL-008 (test infrastructure) should land before most behavior fixes. Phase 0 adds characterization tests only.
- PIN-TPL-001 should precede PIN-TPL-004, because actuator errors must stop being fatal before `.unwrap()` calls are replaced with error propagation.
- PIN-TPL-007 (contract strategy) gates PIN-TPL-006's full fix, PIN-TPL-012 (rediscovery command), PIN-TPL-013 (handshake) and PIN-TPL-017 (`""`→`null`).
- PIN-TPL-003 should be resolved before any actuator is enabled in the default image.

## History

| Date | Change |
|---|---|
| 2026-10-31 | Register created by the initial audit (26 findings, all Open). |
| 2026-10-31 | Phase 0 completed: PIN-TPL-008, -015, -016, -018, -020, -021, -022, -024 moved to In Progress (structural parts done). No finding Resolved. Source paths moved; see the path map in phases/phase-0/RESULT.md §2 (line numbers in records still refer to baseline `5b5c8ed4`). |
| 2026-10-31 | Post-Phase-0 reassessment: all 26 findings re-verified at their new locations (each record now has a "Current location" row). PIN-TPL-016 re-rated Medium → Low (record moved to low.md; pointer kept in medium.md). New finding PIN-TPL-027 (Low). PIN-TPL-024 extended (stale `fake-lidar` README statement). No other severity changes. |
| 2026-10-31 | Phase 1A completed: PIN-TPL-008 stays In Progress with narrowed scope (host harness for hardware-free firmware code, `just test`, CI workflow added; module state machines, dispatch, test type-checking and a first CI run remain). No other changes. |
| 2026-10-31 | Phase 1B completed: PIN-TPL-001, -002 and -025 **Resolved** (each becomes Verified after the hardware check). Totals: 16 Open, 8 In Progress, 3 Resolved. |
| 2026-10-31 | Phase 1C completed: PIN-TPL-006 **Resolved**; PIN-TPL-003 In Progress (pairing window done, application-level authentication remains for 2B). Totals: 15 Open, 9 In Progress, 4 Resolved. |
| 2026-10-31 | Phase 1D completed: PIN-TPL-004 and -005 **Resolved**. Phase 1 complete. Totals: 13 Open, 9 In Progress, 6 Resolved. |
| 2026-11-01 | Phase 2A (protocol v1): PIN-TPL-007 and -012 **Resolved**; PIN-TPL-017 Open → In Progress; -020 and -021 stay In Progress with 2A progress noted. Totals 9 Open, 10 In Progress, 8 Resolved. |
| 2026-11-02 | Phase 2B (protocol v2, transport robustness and authentication): PIN-TPL-003, -010, -013 and -016 **Resolved**. Totals 7 Open, 8 In Progress, 12 Resolved. |
