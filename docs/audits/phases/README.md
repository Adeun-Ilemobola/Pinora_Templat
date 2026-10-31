# Phase index — Pinora Template

Lifecycle statuses: Proposed · Awaiting Approval · Approved · In Progress · Blocked · Completed — Unverified · Completed — Verified.

Every phase follows the same pattern: approval → work → verification → documentation → **stop**. No phase starts without an explicit user approval recorded here.

| Phase | Purpose | Status | Related findings | Validation | Approval | Git revision | Completed |
|---|---|---|---|---|---|---|---|
| Initial audit | Discover, document and rank the current architecture | Completed — Verified (documentation; see [validation baseline](../evidence/validation-baseline.md)) | all | Static checks + 2 probes; no hardware | Requested by user 2026-10-31 | baseline `5b5c8ed4` (docs not committed) | 2026-10-31 |
| [Phase 0](phase-0/PLAN.md) | Structural reorganization without behavior change; characterization tests | **Awaiting Approval** | PIN-TPL-008, -015, -016, -018, -020, -021, -022, -024, -026 (structural parts only) | Planned V1–V10 | — | — | — |
| Post-Phase-0 reassessment | Re-audit the reorganized code; build the remediation roadmap (1A, 1B, …) | Proposed | all | — | — | — | — |

Proposed later phases (indicative only; the real roadmap comes from the reassessment): **1A** firmware command safety (-001, -002, -004, -005, -025) · **1B** trust boundaries (-003, -006) · **1C** test seams and CI (-008) · **2A** protocol contract and identity (-007, -012, -013, -017) · **2B** scheduling and backpressure (-009, -010) · **2C** desktop performance and build (-011, -014, -015, -018, -020) · **3A** inactive sensor modules (-019) · **3B** packaging and cleanup (-023, -026). No directories exist for these yet, and none will be created until the roadmap is approved.
