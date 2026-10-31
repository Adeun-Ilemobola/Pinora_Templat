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
| Phase 1B | Firmware command safety | **Awaiting Approval** | 001, 002, 025 | — | — | — | — |
| Phases 1C, 1D, 2A, 2B, 2C, 2D, 2E, 3A | See [ROADMAP.md](ROADMAP.md) | Proposed | see roadmap | — | — | — | — |

Only `phase-1/phase-1a/` exists. Each later subphase directory is created when that subphase is authorized.