# Audit and modernization changelog — Pinora Template

Newest entries first. Each entry is tagged **[AUDIT]** (documentation only, no source change) or **[SOURCE]** (template source, build or config changed).

## 2026-10-31 — Phase 1C, Trust boundaries **[SOURCE]**

- Approved; no option chosen, so recommended defaults applied (120 s pairing window, `espConsole` event). Branch `phase-1c/trust-boundaries` (local, not pushed), cut from the 1B tip.
- `143f8dac` firmware: pure `pairing_window.rs`; SSP pairing accepted only for 120 s after SPP start, then connectable but non-discoverable; bonded peers still connect.
- `cd1906a6` Tauri: `line_classifier.rs` validates COM lines as Rust `ProtocolMessage`s; others go to the new additive `espConsole` event.
- `9c503b41` UI: total `describeLog`/`matchesLog`; per-route `RouteError` boundary; `espConsole` listener.
- `f8f11b6c`, `8afd9a02` docs (Bluetooth guide, READMEs, architecture, contract notes C-04/C-07) and CI (Tauri unit tests).
- Verification C-V1…C-V5 pass: 53 host, 4 Tauri, 22 frontend tests; golden unchanged; firmware +772 B text; mutation check caught all three reverted fixes. Hardware check C-V6 not run.
- Register: PIN-TPL-006 **Resolved**; PIN-TPL-003 In Progress (authentication remains for 2B). Totals 15 Open, 9 In Progress, 4 Resolved.

## 2026-10-31 — Phase 1B, Firmware command safety **[SOURCE]**

- Approved with decision: clamp out-of-range LED values. Branch `phase-1b/command-safety` (local, not pushed), cut from the 1A tip.
- `bc181a67` overflow-safe `range_*` (`i128` + saturate) and `pulse_us_to_tick` (`i64`); equivalence-tested against the old formulas.
- `ce51ed59` LED clamp to 0–100 with a Medium `SysLog`; `Brightness` reports the applied level; pure `led_mapping.rs`.
- `c0f58fbe` `runtime/dispatch.rs` (handler errors → High `SysLog`, loop continues) and `runtime/tick_errors.rs` (rate-limited Medium `SysLog`); `run() -> !`.
- `939e532b` servo and stepper limit events report stored values.
- `9551123c` docs and contract notes C-13/C-16.
- Verification B-V1…B-V5 pass: 46 host tests (debug and release), golden unchanged, firmware builds (+3 808 B text), mutation check caught all four reverted fixes. Hardware check B-V6 not run.
- Register: PIN-TPL-001, -002, -025 **Resolved**. Totals 16 Open, 8 In Progress, 3 Resolved. PIN-TPL-001 removed from the Critical watch-list.

## 2026-10-31 — Phase 1A, Firmware host-test harness and CI **[SOURCE]**

- Roadmap approved (scope and order) together with Phase 1A. Branch `phase-1a/host-test-harness` (local, not pushed), cut from the Phase 0 tip.
- `6f22bd2a` reassessment docs committed; `3dc86120` 1A plan.
- `fbc53466` `tests/firmware-host/`: host crate compiling `math.rs`, `core/module.rs`, `core/transport/emitter.rs`, `discovery_cache.rs` unchanged; 28 characterization tests.
- `8e611b3e` `just test` recipes and `bun run test` script.
- `3fef4539` `.github/workflows/ci.yml` (ubuntu Rust job, windows desktop job); type alias in the protocol golden test (test code only).
- Verification A1–A7 pass: mutation check caught both injected faults; firmware ELF sizes identical to Phase 0; no firmware, protocol, backend or frontend source changes. CI not yet executed on GitHub.
- Register: PIN-TPL-008 stays In Progress (scope narrowed). No other changes.

## 2026-10-31 — Post-Phase-0 reassessment and roadmap **[AUDIT]**

- Re-verified all 26 findings in the reorganized tree. Every record gained a "Current location (after Phase 0)" row; baseline line references are kept.
- No new problems introduced by Phase 0. New pre-existing finding **PIN-TPL-027** (Low: randomized module order). **PIN-TPL-016** re-rated Medium → Low (record moved to `low.md`, pointer left in `medium.md`). PIN-TPL-024 extended (`fake-lidar` README statement).
- Totals: 0 Critical, 6 High, 13 Medium, 8 Low (27); 19 Open, 8 In Progress.
- Added `reports/post-phase-0-reassessment.md` and `phases/ROADMAP.md` (1A–1D, 2A–2E, 3A, with full subphase records). Refreshed `architecture/current-system.md` and `module-boundaries.md` for the new layout.
- **Source changes: none.** The documentation changes are not committed.

## 2026-10-31 — Phase 0, Structural Reorganization **[SOURCE]**

- Approved with decisions D1–D3 yes, D4 no, D5–D6 yes. Work is on local branch `phase-0/structural-reorganization` (not pushed; `main` unchanged).
- `693e4e05` P0.1: characterization tests (protocol golden JSON, 108 cases; Bun tests for log helpers, 16).
- `7fef0c4f` P0.2: rustfmt on all three Rust packages.
- `2b44cf80` P0.3: firmware reorganized into `board/`, `runtime/`, `core/{module,hardware/*,transport/*}`, `modules/{actuators,sensors,composite}/`; internal renames.
- `d9a2e37d` P0.4: Tauri backend split into `commands`, `serial_transport`, `bluetooth_ports`.
- `13f0abc9` P0.5: frontend module files split into schema/store/view; `Modulefront.TS` → `.ts`.
- `a4e5b282` P0.6: module docs, `// SAFETY:` notes, project-doc path updates.
- `de994a70` P0.7: deleted `core/emitter.rs` and `ModuleGeter.ts` (D3).
- Verification V1–V10 all pass (see `phases/phase-0/RESULT.md`). No hardware test. 8 findings moved to In Progress; none Resolved.
- Docs: `phases/phase-0/RESULT.md` added; register, README, phase index and plan status updated; mirror re-synchronized.

## 2026-10-31 — Initial audit **[AUDIT]**

- Created the `docs/audits/` tree: README, architecture (4 documents), findings register (26 findings: 0 Critical, 6 High, 14 Medium, 6 Low), phase index, Phase 0 plan, initial-audit report, validation baseline and probe sources.
- Ran hardware-independent checks with build output isolated outside the repository. Protocol and Tauri `check`/`clippy` pass; `tsc --noEmit` passes; `cargo fmt --check` fails (pre-existing). Firmware build not run (needs approval D5).
- Ran two throwaway probes outside the repository against real sources (protocol wire shapes and LED arithmetic; desktop log helpers).
- Mirrored all of the above to `C:\Dev\Pinora\Audits\Pinora_Templat\` and created `C:\Dev\Pinora\Audits\README.md`.
- **Source changes: none.** The template working tree was unchanged apart from the new `docs/audits/` directory (later committed as `cb773d3b` on the Phase 0 branch).
- Existing `docs/architecture-audit.md` was left untouched; the new tree sits alongside it.
