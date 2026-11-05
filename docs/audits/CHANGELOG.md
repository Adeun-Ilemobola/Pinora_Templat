# Audit and modernization changelog — Pinora Template

Newest entries first. Each entry is tagged **[AUDIT]** (documentation only, no source change) or **[SOURCE]** (template source, build or config changed).

## 2026-11-05 — Floating-state audit (PIN-TPL-028 to -036) **[SOURCE]**

- The maintainer asked for a search for values kept apart from what really happened. Nine findings, all fixed: servo state and LiDAR scans (028-030), RFID writes (031), replay and invented desktop values (032), desktop stores and the LiDAR page (033), stepper reports and homing (034), button and joystick (035), misleading logs (036).
- Host tests 146 (18 new), frontend 173 (6 new), mutations caught, firmware builds with no warnings. Only the LED image has run on hardware; none of the changed modules has.

## 2026-11-05 — LiDAR scan follows the servos (PIN-TPL-005 corrected) **[SOURCE]**

- Found by the maintainer: the 1D `ScanPlan` kept its own position and never read the servos, so a clamped or moved servo disagreed with the plan.
- `advance(current)` now takes the position read from the servos and the plan holds none of its own; `Lidar::tick` reads it just before recording. A servo that stays put when asked to move ends the row (x) or the scan (y), so scans stay finite. A clamped x axis records its last position twice per row.
- Host tests 128 (6 new, fake clamping, moved and stuck servos, 4 500 clamped combinations); three mutations caught; firmware builds with no warnings. Not run on hardware.

## 2026-11-02 — CI green **[SOURCE]**

- `c7d6ae55`: host-test crate builds on Linux (flat `#[path]` includes); `actions/checkout` v5.
- `f2f82d9`: clippy-clean on Rust 1.99 (`as_chunks`; compare-exchange instead of the deprecated `fetch_update`).
- CI run 41532290661: both jobs pass. Local backups, `refs/original` and the affected reflogs removed and objects pruned; the stash was kept.

## 2026-11-02 — Pushed **[AUDIT]**

- Before the first push, 6 399 Cargo build files under `UI/src-tauri/debug/` (committed by mistake in Phase 3A, including a generated copy of the local `wifi_config.rs`) were removed from the six unpushed commits with your approval. Contents are otherwise identical; backups were kept locally. The folder is now ignored.
- `main` was pushed to GitHub as a fast-forward from `5b5c8ed4`; the CI workflow started.

## 2026-11-02 — Merged into `main` **[AUDIT]**

- At your request, `main` was fast-forwarded from `5b5c8ed4` to `phase-3a/cleanup` (`cf61f79b`, after the history fix below): 67 commits, no merge commit, no rewritten history. Phase branches kept. Not pushed.

## 2026-11-02 — Phase 3A, Low-severity cleanup and hardening **[SOURCE]** — roadmap complete

- Branch `phase-3a/cleanup`, cut from the 2E tip. Identifier kept; `.espConfig` and `pre-script.rhai` untouched (C-10).
- `726e499f`, `ef6a2bc3`: zero warnings; clippy `-D warnings` in CI; CSP; product name; `greet` removed; `start_port` error detail.
- `e6a62d9c`: `p.uart0` consumed; `"log"` SysLogs handled; documentation drift fixed. `e9d772bd`: probe clippy.
- Verified: all suites, zero warnings, hw-test 23/23 + 5/5 on the 3A image.
- Register: 022, 023, 024, 026 **Resolved**; 017, 021 **Blocked** (wire-breaking). Final totals 0 Open, 0 In Progress, 2 Blocked, 25 Resolved.

## 2026-11-02 — Phase 2E, Build reproducibility and hardware configuration **[SOURCE]**

- Branch `phase-2e/reproducible-builds`, cut from the 2D tip. GPIO12 kept (your wiring); `C:/t` kept with a `CARGO_TARGET_DIR` override.
- `8addc1ad` ESP-IDF Git patches pinned by `rev`; `rand` removed; `embassy-time` kept (required at link time; the audit's "unused" note was wrong).
- `2435464d` `wifi_config` fallback in `protocol/build.rs`; CI no longer copies the example; `board/pin_map.rs` with host checks.
- A fresh clone (offline, no `wifi_config.rs`) built and passed protocol, host, frontend and Tauri suites; release ELF unchanged; hw-test 23/23 + 5/5.
- Register: 015, 018 **Resolved**. Totals 2 Open, 4 In Progress, 21 Resolved.

## 2026-11-02 — Phase 2D, Desktop performance and structure **[SOURCE]**

- Branch `phase-2d/desktop`, cut from the 2C tip. Defaults: 5 000-entry history with export, 50 ms batches, warn/error-only console forwarding, 10 s port cache.
- `80f71533` bounded, batched logs; `e2317af4` async slow commands and port cache; `5d50de28` store split, strict typing, store characterization tests.
- bun test 166, Tauri 21; mutation check caught all three removals. Measured port query: 0.43–0.91 s warm, ~7 s cold.
- Register: 011, 014, 020 **Resolved**. Totals 2 Open, 6 In Progress, 19 Resolved.

## 2026-11-02 — Phase 2C, Firmware runtime and sensor modules **[SOURCE]**

- Authorized under your blanket approval; defaults recorded in the plan (1 kHz tick and blocking 1 ms loop, RFID record v1, RMT IR capture). Branch `phase-2c/runtime-sensors`, cut from the 2B tip.
- `8d6c00f4` sensors: pure NEC decoder and RMT capture; RFID record format, bounded write buffer, reader-version check.
- `2ce4dd2c` insertion-ordered module registry.
- `49929faa` scheduler: 1 ms period, wait compensation for ESP-IDF's extra tick, loop statistics; `CONFIG_FREERTOS_HZ=1000`.
- Measured on the board: the first version ran at 1.83 ms (ESP-IDF rounds timed waits up and adds a tick); after the fix, mean 1.000 ms, p99 ≤ 1.2 ms, 0 overruns. Registration order fixed across boots. hw-test 23/23 and 5/5.
- Host tests 118. Register: 008, 009, 019, 027 **Resolved**. Totals 4 Open, 7 In Progress, 16 Resolved.

## 2026-11-02 — Phase 2B, Transport robustness and authentication **[SOURCE]**

- Approved 2026-11-01 with decisions: RustCrypto in the protocol crate, pairing over USB, read-only enforcement. Branch `phase-2b/transport-auth` (local, not pushed), cut from the 2A tip.
- `b126a1d7` protocol v2: one line framer and limit table; HMAC handshake and session state machines; signed commands; golden gains 11 backend-only lines.
- `dcde0423` firmware: shared framing, counted drops with a 5 s report, rate limits, Wi-Fi background bring-up and reader/writer split.
- `e07262e9` firmware authentication: NVS key, USB console in every image, per-connection sessions.
- `353de798`, `d221b6df` desktop: Auth lines never reach the WebView; key file; authenticated, signing COM and Wi-Fi transports; listener deadline and replacement; new IPC commands and `deviceAuth`.
- `3b616771` frontend: Device security card with USB pairing.
- Docs: contracts C-01…C-03, C-05…C-08, C-15, C-17, C-19; communication flow; guides; `tests/hardware-probe` and `just hw-test`.
- On 2026-11-02 you authorized on-device testing and the remaining phases. The board was flashed (project bootloader and partitions) and passed 23/23 protocol checks and 5/5 key-persistence checks over USB and Bluetooth, including the 1B clamp and the 2A reconnect replay.
- Verification B2-V1…B2-V6 pass: 31 protocol, 97 host, 19 Tauri, 144 frontend tests; five mutations caught (one after strengthening a test). Release image −101 KB text (unused Wi-Fi stack now removed by LTO, probable).
- Register: PIN-TPL-003, -010, -013, -016 **Resolved**. Totals 7 Open, 8 In Progress, 12 Resolved. Critical watch-list empty.

## 2026-11-01 — Phase 2A, Protocol contract v1 **[SOURCE]**

- Approved; no choice made on misspelled wire names, so the conservative default applied (output kept, corrected spellings accepted as input aliases). Branch `phase-2a/protocol-v1` (local, not pushed), cut from the 1D tip.
- `c6b31f38` protocol v1: `PROTOCOL_VERSION = 1`, `SystemInfo.protocol_version` (pre-v1 parses as 0), `System(Rediscover {})` on `"SSI"`, input aliases. Golden diff: one new line, one new field.
- `03d43559` firmware: rediscovery re-sends snapshot, registrations and current state (`first_emit`); child routing via `Module::child_mut`; duplicate lookup names reported.
- `de71ad3d` Tauri: additive `serialConnection` event on USB read errors.
- `fd551fe1`, `d7b5046d` frontend: complete Zod schemas checked against all 109 golden lines; validation on receive and send; clear-before-connect; `Rediscover` after connect; USB loss shown; LED unknown until reported; protocol version and duplicate-name warnings. `d7b5046d` restores LF line endings only.
- Docs: contract notes C-01, C-05, C-06, C-07; communication flow; discovery-cache doc comment.
- Verification A-V1…A-V5 pass: 6 protocol, 79 host, 5 Tauri, 137 frontend tests; firmware warnings 7 → 6; release +10 116 B text; mutation check caught all four removals. Hardware check A-V6 not run.
- Register: PIN-TPL-007 and -012 **Resolved**; -017 In Progress. Totals 9 Open, 10 In Progress, 8 Resolved. Correction: the 1D entry and README said 13 Open; the register had 12.

## 2026-10-31 — Phase 1D, Actuator and LiDAR safety **[SOURCE]**

- Approved; no option chosen, so the recommended default applied (coils released when idle, opt-in holding). Branch `phase-1d/actuator-safety` (local, not pushed), cut from the 1C tip.
- `5aec9215` stepper: pure `stepper_motion.rs` core; starts Idle; targets bounded (±180° travel, pivot `min <= max`); absolute pivot targets (the old relative targets ran away, reproduced); origin and pivot moves complete; coils released when idle; coil errors propagate.
- `4123676b` LiDAR: pure `lidar_scan.rs` planner; ROI and step validated; clamped stepping guarantees termination (the old exact-equality check did not, reproduced); buffer flushed every 100 points; correct `max_chunk`; `first_emit` no longer moves.
- `1e1e5a63` servo: construction writes no PWM; explicit `run_self_test()`; no `unwrap()`.
- `80bf102f` docs and contract notes C-13/C-16.
- Verification D-V1…D-V4 pass: 72 host tests (3 240 scan combinations), mutation check caught all four reverted fixes, no hardware writes in constructors, firmware warnings 11 → 7, active-image ELF unchanged. Supervised hardware check D-V5 not run.
- Register: PIN-TPL-004 and -005 **Resolved**; only PIN-TPL-003 remains on the Critical watch-list. **Phase 1 complete.** Totals 13 Open, 9 In Progress, 6 Resolved.

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
