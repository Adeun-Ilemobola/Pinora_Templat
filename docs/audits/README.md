# Pinora Template — Audit & Modernization Record

> **Authoritative copy.** This directory (`Pinora_Templat/docs/audits/`) is the source of truth. A synchronized mirror lives at `C:\Dev\Pinora\Audits\Pinora_Templat\`.

| | |
|---|---|
| **Project** | Pinora Template (`Pinora_Templat`, v0.8.0-alpha) |
| **Scope** | `C:\Dev\Pinora\Pinora_Templat` only. `2dLidar`, `pinora_cli`, `pinora_studio` and `UI` are **not audited** |
| **Audit date** | 2026-10-31 |
| **Baseline** | `main` @ `5b5c8ed4`, clean working tree |
| **Current operation** | Phase 0, Structural Reorganization |
| **Current stage** | Phase 0 **Completed — Verified** (hardware-independent V1–V10; hardware smoke test not done). See [RESULT](phases/phase-0/RESULT.md) |
| **Approval status** | ⏸ **Stopped for review.** Next: the post-Phase-0 reassessment and roadmap, which needs its own approval |
| **Source changes so far** | Phase 0 structural changes on local branch `phase-0/structural-reorganization` (not pushed; `main` unchanged at `5b5c8ed4`) |
| **Last synchronization** | 2026-10-31: mirror verified by SHA-256 over all files (see the mirror's `SYNC-STATUS.md`) |

## Severity totals

| Critical | High | Medium | Low | Total |
|:-:|:-:|:-:|:-:|:-:|
| **0** | **6** | **14** | **6** | **26** |

18 Open · 8 In Progress (structural parts done in Phase 0) · 0 Resolved. Functional defects are untouched. Four High findings are on the [Critical escalation watch-list](findings/critical.md): they become Critical as soon as actuator or LiDAR code is enabled.

## Most important risks

1. **Firmware stops obeying after one failed command.** It keeps running but ignores everything until rebooted (PIN-TPL-001).
2. **Anyone nearby can pair over Bluetooth and send commands** (PIN-TPL-003).
3. **An out-of-range LED value reboots debug builds or mis-drives the LED** (PIN-TPL-002, shown by probe).
4. **Dormant motor code moves hardware on boot and accepts unlimited travel** (PIN-TPL-004). The **LiDAR scan may never finish** and exhausts memory (PIN-TPL-005).
5. **One malformed line from the board can crash the desktop UI** (PIN-TPL-006, shown by probe).
6. **No tests and a hand-copied protocol** make every change risky (PIN-TPL-007, PIN-TPL-008).

## Issue summary

| ID | Sev | Title | Status |
|---|---|---|---|
| PIN-TPL-001 | High | One failed command stops the firmware main loop | Open |
| PIN-TPL-002 | High | LED brightness unchecked; overflow corrupts output or reboots | Open |
| PIN-TPL-003 | High | Bluetooth accepts any nearby device; no app authorization | Open |
| PIN-TPL-004 | High | Actuators move uncommanded; unbounded targets *(latent)* | Open |
| PIN-TPL-005 | High | LiDAR scan never terminates; unbounded memory *(latent)* | Open |
| PIN-TPL-006 | High | Malformed device input crashes the desktop log view | Open |
| PIN-TPL-007 | Med | Rust↔TS contract handwritten, partial, unversioned | Open |
| PIN-TPL-008 | Med | No automated tests or CI | In Progress |
| PIN-TPL-009 | Med | Main loop busy-polls; no timing guarantees | Open |
| PIN-TPL-010 | Med | Telemetry backpressure and silent drops | Open |
| PIN-TPL-011 | Med | Frontend log history unbounded, copied per message | Open |
| PIN-TPL-012 | Med | Connection-state desync; no state replay | Open |
| PIN-TPL-013 | Med | Wi-Fi dev-grade auth; fragile boot dependency | Open |
| PIN-TPL-014 | Med | Tauri commands block the UI thread | Open |
| PIN-TPL-015 | Med | Fresh clone does not build; machine-specific config | In Progress |
| PIN-TPL-016 | Med | Duplicated logic and dead code | In Progress |
| PIN-TPL-017 | Med | Module identity and hierarchy incomplete | Open |
| PIN-TPL-018 | Med | Hard-coded pins; LED on strapping pin GPIO12 | In Progress |
| PIN-TPL-019 | Med | Defects in inactive RFID, IR and rangefinder modules | Open |
| PIN-TPL-020 | Med | Frontend god-store, oversized files, `any` | In Progress |
| PIN-TPL-021 | Low | Misspellings (wire names frozen, internal fixable) | In Progress |
| PIN-TPL-022 | Low | Formatting and lint not enforced | In Progress |
| PIN-TPL-023 | Low | Tauri scaffold metadata, CSP off, leftovers | Open |
| PIN-TPL-024 | Low | Documentation drift | In Progress |
| PIN-TPL-025 | Low | Events report requested, not applied values | Open |
| PIN-TPL-026 | Low | UART0 driver bypasses HAL ownership | Open |

Full records: [findings/](findings/README.md).

## Outstanding problems and next step

- **Done:** Phase 0, Structural Reorganization ([plan](phases/phase-0/PLAN.md), [result](phases/phase-0/RESULT.md)). Decisions: D1–D3 yes, D4 no, D5–D6 yes. One local commit per step (P0.1–P0.7) on `phase-0/structural-reorganization`; all checks V1–V10 pass; no contract, manifest, lockfile or config changed.
- **Your review:** inspect the branch, optionally run the hardware smoke test in RESULT §8, and decide whether to merge it into `main`. Merging and pushing are your decisions.
- **Proposed next step:** post-Phase-0 reassessment and remediation roadmap (Phase 1A, 1B, …). **Not started**; needs your approval.
- **Validation gaps:** no hardware or runtime verification yet. Firmware module logic has no host tests (PIN-TPL-008).

## Documents

| Area | Document |
|---|---|
| Architecture as implemented | [architecture/current-system.md](architecture/current-system.md) |
| Module boundaries | [architecture/module-boundaries.md](architecture/module-boundaries.md) |
| Communication flow | [architecture/communication-flow.md](architecture/communication-flow.md) |
| Frozen contracts | [architecture/compatibility-contracts.md](architecture/compatibility-contracts.md) |
| Issue register | [findings/README.md](findings/README.md) · [critical](findings/critical.md) · [high](findings/high.md) · [medium](findings/medium.md) · [low](findings/low.md) |
| Phases | [phases/README.md](phases/README.md) · [Phase 0 plan](phases/phase-0/PLAN.md) · [Phase 0 result](phases/phase-0/RESULT.md) |
| Report | [reports/initial-audit.md](reports/initial-audit.md) |
| Evidence | [evidence/validation-baseline.md](evidence/validation-baseline.md) · [evidence/probes.md](evidence/probes.md) |
| Changelog | [CHANGELOG.md](CHANGELOG.md) |

## Version control

- `docs/audits/` is committed on the local branch `phase-0/structural-reorganization` (first commit `cb773d3b`); it is not on `main` until you merge.
- The workspace mirror `C:\Dev\Pinora\Audits\` is **not under version control**: `C:\Dev\Pinora` is not a Git repository, and no repository was created.
- Pre-existing `docs/architecture-audit.md` (an earlier review) was left unchanged.

## Conventions

- Issue IDs `PIN-TPL-NNN` are permanent. Findings are never deleted; status moves Open → Planned → In Progress → Resolved → Verified (or Blocked).
- Phase statuses: Proposed · Awaiting Approval · Approved · In Progress · Blocked · Completed — Unverified · Completed — Verified.
- Source references are repository-relative paths with line numbers at the stated baseline commit, so these documents stay valid when copied elsewhere.
