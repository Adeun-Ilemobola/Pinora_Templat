# Phase index — Pinora Template

Lifecycle statuses: Proposed · Awaiting Approval · Approved · In Progress · Blocked · Completed — Unverified · Completed — Verified.

Every phase follows the same pattern: approval → work → verification → documentation → **stop**. No phase starts without an explicit user approval recorded here.

| Phase | Purpose | Status | Related findings | Validation | Approval | Git revision | Completed |
|---|---|---|---|---|---|---|---|
| Initial audit | Discover, document and rank the current architecture | Completed — Verified (documentation; see [validation baseline](../evidence/validation-baseline.md)) | all | Static checks + 2 probes; no hardware | Requested by user 2026-10-31 | baseline `5b5c8ed4`; docs committed as `cb773d3b` | 2026-10-31 |
| [Phase 0](phase-0/PLAN.md) ([result](phase-0/RESULT.md)) | Structural reorganization without behavior change; characterization tests | **Completed — Verified** (hardware-independent V1–V10; hardware smoke test not done) | PIN-TPL-008, -015, -016, -018, -020, -021, -022, -024, -026 (structural parts) | V1–V10 pass | Approved 2026-10-31 (D1–D3 yes, D4 no, D5–D6 yes) | `693e4e05`…`de994a70` on `phase-0/structural-reorganization` (local) | 2026-10-31 |
| Post-Phase-0 reassessment | Re-audit the reorganized code; build the remediation roadmap | Completed — Verified (documentation; see [report](../reports/post-phase-0-reassessment.md)) | all | Evidence re-located for all 26 findings; 1 new finding | Approved 2026-10-31 | basis `a2204b33` (docs uncommitted) | 2026-10-31 |
| [Roadmap](ROADMAP.md) | Ordered remediation scope (10 subphases) | **Approved** (scope and order) | all 27 | — | Approved 2026-10-31 | `6f22bd2a` | — |
| [Phase 1A](phase-1/phase-1a/PLAN.md) ([result](phase-1/phase-1a/RESULT.md)) | Firmware host-test harness and CI | **Completed — Verified** (A1–A7; CI not yet run on GitHub) | 008 | 28 host tests; mutation check; firmware ELF unchanged | Approved 2026-10-31 | `fbc53466`…`3fef4539` on `phase-1a/host-test-harness` (local) | 2026-10-31 |
| [Phase 1B](phase-1/phase-1b/PLAN.md) ([result](phase-1/phase-1b/RESULT.md)) | Firmware command safety | **Completed — Verified** (hardware-independent B-V1…B-V5; hardware check B-V6 pending) | 001, 002, 025 → Resolved | 46 host tests; mutation check; golden unchanged | Approved 2026-10-31 (clamp) | `bc181a67`…`9551123c` on `phase-1b/command-safety` (local) | 2026-10-31 |
| [Phase 1C](phase-1/phase-1c/PLAN.md) ([result](phase-1/phase-1c/RESULT.md)) | Trust boundaries | **Completed — Verified** (hardware-independent C-V1…C-V5; hardware check C-V6 pending) | 006 → Resolved; 003 → In Progress | 53 host + 4 Tauri + 22 frontend tests; mutation check | Approved 2026-10-31 (defaults: 120 s, espConsole) | `143f8dac`…`8afd9a02` on `phase-1c/trust-boundaries` (local) | 2026-10-31 |
| Phase 1D | Actuator and LiDAR safety | **Awaiting Approval** | 004, 005 | — | — | — | — |
| Phases 2A, 2B, 2C, 2D, 2E, 3A | See [ROADMAP.md](ROADMAP.md) | Proposed | see roadmap | — | — | — | — |

Only `phase-1/phase-1a/`, `phase-1/phase-1b/` and `phase-1/phase-1c/` exist. Each later subphase directory is created when that subphase is authorized.