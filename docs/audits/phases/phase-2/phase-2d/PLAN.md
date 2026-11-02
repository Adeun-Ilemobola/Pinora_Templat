# Phase 2D — Desktop performance and structure (PLAN)

| Field | Value |
|---|---|
| Phase | 2D |
| Status | **Completed — Verified** 2026-11-02; see [RESULT.md](RESULT.md). This plan was written alongside the work (you authorized all remaining phases without gates); its scope is the roadmap's |
| Decisions (defaults) | Log history capacity **5 000** entries with JSON export; flush batches every **50 ms**; console forwarding limited to warnings and errors; Bluetooth port query cached **10 s** (Refresh always re-queries); `send_data` stays synchronous to keep commands ordered |
| Branch | `phase-2d/desktop`, cut from `phase-2c/runtime-sensors` @ `a6c38afe` (local, not pushed) |
| Linked findings | PIN-TPL-011, PIN-TPL-014, PIN-TPL-020 |
| Wire format / IPC | Unchanged names and arguments (C-01, C-06, C-07) |

## Changes

| Step | Change | Findings |
|---|---|---|
| D1 | Bounded, batched log history; cached summaries and search text; export; no per-message `console.info`; console forwarding only for warn/error | 011 |
| D2 | Slow Tauri commands `async` on blocking threads; Bluetooth port query cached | 014 |
| D3 | Store split into transport, registry, logs and auth slices; `noImplicitAny`; no explicit `any`; store characterization tests (written first) | 020 |

## Verification

| # | Check | Pass criterion |
|---|---|---|
| D-V1 | 100 000 synthetic messages | History bounded at capacity; append cost flat |
| D-V2 | Store characterization tests | Pass before and after the split |
| D-V3 | Port query timing | Measured; cached calls near zero |
| D-V4 | `tsc`, Vite build, `bun test`, Tauri tests and clippy | Green |
| D-V5 | Mutation | Removing the cap, the log push or the rediscovery makes tests fail |
