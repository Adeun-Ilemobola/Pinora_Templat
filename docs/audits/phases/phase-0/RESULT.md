# Phase 0 — Structural reorganization (RESULT)

| Field | Value |
|---|---|
| Phase | 0 |
| Status | **Completed — Verified** against the hardware-independent criteria V1–V10. A hardware smoke test is **not** done (see §8) |
| Approved | 2026-10-31, by the user, with decisions D1 yes, D2 yes, D3 yes, D4 no (default), D5 yes, D6 yes |
| Executed | 2026-10-31 |
| Branch | `phase-0/structural-reorganization` (local only; **not pushed**; `main` untouched at `5b5c8ed4`) |
| Baseline | `5b5c8ed4` |
| Result revision | `de994a70` (last code commit; this report is committed on top of it) |
| Behavior change intended | None. Two accepted cosmetic differences (D6), see §6 |

## 1. Commits (one per step, oldest first)

| Step | Commit | Content |
|---|---|---|
| — | `cb773d3b` | Initial audit record (`docs/audits/`) committed to the branch |
| P0.1 | `693e4e05` | Characterization tests: protocol golden JSON (108 cases) and Bun tests for the log helpers (16) |
| P0.2 | `7fef0c4f` | `cargo fmt` on protocol, firmware and Tauri (formatting only) |
| P0.3 | `2b44cf80` | Firmware reorganization and internal renames |
| P0.4 | `d9a2e37d` | Tauri backend split |
| P0.5 | `13f0abc9` | Frontend module split, `Modulefront.TS` → `.ts`, registry helper |
| P0.6 | `a4e5b282` | Doc comments, `// SAFETY:` notes, project-doc path updates |
| P0.7 | `de994a70` | Deleted two unreferenced dead files (D3) |

## 2. Original vs new organization

### Firmware (`Firmware_Templates/src`)

| Before | After |
|---|---|
| `main.rs` (297 lines: UART setup, all wiring, ~130 lines of commented wiring, main loop) | `main.rs` (startup order only, 74 lines) · `board/mod.rs` (active LED wiring and pin map) · `board/disabled_wiring.rs` (commented wiring, verbatim, with hazard notes) · `runtime/mod.rs` (`register_all`, main loop) |
| `core/hardware.rs` | `core/hardware/{mod,pins,timing,context,system_info,console}.rs` (`mod.rs` re-exports the old paths) |
| `core/modulecore.rs` | `core/module.rs` (+ `ModuleHandle`, `ModuleRegistry` type aliases) |
| `core/transport/transport_core.rs` | `core/transport/manager.rs`, with the Serial reader and writer moved to `core/transport/serial.rs` |
| `core/transport/transport_emiter.rs` | `core/transport/emitter.rs` |
| `remember()` duplicated in `wifi.rs` and `bluetooth.rs` | `core/transport/discovery_cache.rs` (one copy) |
| `core/emitter.rs` (not compiled) | deleted (D3) |
| `module/{ledmodule,servomodule,stepper}.rs` | `modules/actuators/{led,servo,stepper}.rs` |
| `module/{buttonmodule,range_finder,rfid,remote_receiver,joystick}.rs`, `module/imu/{imu_type,mpu_impl}.rs` | `modules/sensors/{button,rangefinder,rfid,remote_receiver,joystick}.rs`, `modules/sensors/imu/{types,mpu}.rs` |
| `module/lidar.rs` | `modules/composite/lidar.rs` |

### Desktop backend (`UI/src-tauri/src`)

| Before | After |
|---|---|
| `lib.rs` (415 lines) | `lib.rs` (plugin setup and handler list) · `commands.rs` (10 IPC commands) · `serial_transport.rs` (`AppState`, COM reader) · `bluetooth_ports.rs` (Windows PnP query) · `wifi_transport.rs` (unchanged) |

### Desktop frontend (`UI/src`)

| Before | After |
|---|---|
| `lib/Modules/<x>.tsx` (×9, up to 948 lines each) | `lib/Modules/<x>/{schema.ts, store.ts, view.tsx, index.ts}`, with LiDAR also getting `geometry.ts`; importers unchanged through `index.ts` |
| `lib/Modulefront.TS` | `lib/Modulefront.ts` |
| `registrationHasParent` in `Modulefront` | `lib/registry/registration.ts` (re-exported from `Modulefront.ts`) |
| `lib/ModuleGeter.ts` (no importers) | deleted (D3) |

### Tests and docs added

`protocol/tests/wire_format.rs` + `protocol/tests/golden/wire_format.txt`; `UI/tests/logs.test.ts`. The README, `UI/README.md`, `docs/architecture-audit.md` and `Firmware_Templates/BLUETOOTH_TRANSPORT.md` were updated with the new paths and the test commands.

## 3. Renamed internal symbols

`TransportEmiter`→`TransportEmitter` · module `transport_emiter`→`emitter` · `transport_core`→`manager` · `modulecore`→`module` · `ModuleCore::manuel_id`→`lookup_name` (the wire field `lool_up_id` is unchanged) · `Ledmodule`→`LedModule` · `Buttonmodule`→`ButtonModule` · `tansport`→`transport` · `transport_core` (local)→`selected_transport` · `buzer_pin`→`buzzer_pin` · `write_chuck`→`write_chunk` · `testrang`→`self_test_angles` · Button constructor parameter `lool_up_id`→`lookup_name`.
**No protocol-crate name was changed** (C-08).

## 4. Extracted helpers

| Helper | From | Ordering and side-effect notes |
|---|---|---|
| `board::install_active_modules` | `main` LED blocks | Same three blocks, same order; `sync_sender.clone()` per LED as before |
| `runtime::register_all` | `main` registration loop | Same iteration and `?` semantics |
| `runtime::run` | `main` loop | Identical body. `last_yield_us` is still sampled in `main` at the original point and passed in. `command_receiver` is moved in; it was declared last and so dropped first before, and is dropped first now |
| `core::hardware::configure_console_uart` | `main.rs` | Verbatim |
| `transport::serial::{send, command_reader}` | `TransportCore` | Verbatim; `send` takes `&ProtocolMessage` (same serialization) |
| `transport::discovery_cache::remember` | `wifi.rs`, `bluetooth.rs` | The two copies were byte-identical (checked by `diff`) |
| `registry::registrationHasParent` | `Modulefront` | Pure function |

Deliberately **not** extracted: the frontend registration factory map stays inside `RegistrationEvent`, because a module-level map would be evaluated during the existing `Modulefront` ↔ module import cycle.

## 5. Tests executed and results

All checks ran with build output outside the repository, except the firmware cache `C:/t` (D5). Logs are kept with the session's scratch files; the summary is here.

| # | Check | Baseline | Result |
|---|---|---|---|
| V1 | Protocol golden JSON (108 cases) + Bluetooth constants | written on unchanged code (P0.1) | ✅ 2/2 tests pass; golden file **unchanged** since P0.1 |
| V2 | Protocol `check` / `clippy`; Tauri `clippy` | pass / 1 warning; pass / 2 warnings | ✅ pass / 1; ✅ pass / 2 (same warnings) |
| V3 | Firmware `cargo +esp-1.93 build --frozen` debug and release | pass, 17 warnings; release ELF text 1 113 234 / data 250 878 / bss 42 539 | ✅ pass, **11** warnings (6 unused-import warnings in old `main.rs` gone); release ELF text **1 113 390** (+156) / data **250 958** (+80) / bss 42 539 (=) |
| V4 | `tsc --noEmit`; Vite build | pass; 6 911 modules, JS 644 982 B | ✅ pass; 6 918 modules (+7 new split files reachable), JS **644 982 B (identical size)** |
| V5 | `bun test` characterization | 16 pass (written on unchanged code) | ✅ 16 pass, 0 fail |
| V6 | Contract strings (C-03, C-04, C-05, C-07, C-12, C-13) | — | ✅ All present. Count changes come only from new doc comments and parameter names, plus the deleted dead emitter (`sync_channel` 7→6) |
| V7 | Line-level move check | — | ✅ Normalized multiset diff of all firmware code (comments, `use`/`mod` and whitespace ignored; approved renames applied): the 38 removed / 66 added lines are all helper signatures and calls, import paths, the de-duplicated `remember()` copy, or local renames. Frontend: each split module re-concatenated equals the original body (9/9 verbatim) |
| V8 | Startup order (C-15) | `main.rs:58-295` | ✅ Same statement sequence: patches → logger → peripherals → console UART → event loop → NVS → transport → system info → registry → `last_yield_us` → I²C → hardware → LEDs 1, 2, 3 → register/first_emit → command receiver → loop |
| V9 | Manifests, lockfiles, configs, `pinora.toml`, `.espConfig`, `pre-script.rhai`, `justfile`, `tsconfig`, `vite.config`, `tauri.conf.json`, capabilities, `.gitignore` | — | ✅ **No diff** against `5b5c8ed4` |
| V10 | `cargo fmt --check` (3 packages) | fail (78/21/6 hunks) | ✅ clean |

### Binary-level differences (explained)

- **Firmware.** The release ELF is not byte-identical, and is not expected to be. Per-symbol code sizes are identical except for four symbols: `main` (−115 B), the sender-thread closure (−47 B), a now out-of-line `drop_in_place::<TransportCore>` (+95 B), and `serial::send` (+201 B). These are inlining differences caused by the helper extraction. `.flash.rodata` grew 80 B and `.flash.text` 156 B, from longer module-path and panic-location strings. `.dram0.data` and `.bss` are unchanged.
- **Frontend.** The bundle is the same size. Differences are minifier-assigned identifiers and module order only.

## 6. Compatibility notes

- **Accepted (D6).** Firmware console log targets now print the new module paths, for example `Firmware_Templates::core::transport::manager` instead of `…::transport_core`. Panic file and line strings changed too. Neither is parsed by the desktop.
- `lidar/geometry.ts` now exports `Clamp` and `GridCells`, which were module-private before, because `view.tsx` uses them. They are reachable through `lib/Modules/lidar`. No behavior changes.
- Frontend import paths are unchanged: `./Modules/<x>` now resolves to `./Modules/<x>/index.ts`.
- **Line endings.** The repository mixes LF and CRLF. Edited files keep their original line endings; new files use LF. No `.gitattributes` was added, since that would be a configuration change.
- **Finding line references.** Findings cite line numbers at baseline `5b5c8ed4`. Use the path map in §2 to locate the same code now.

## 7. Effect on findings

Status changes only. The records are kept in the register.

| Finding | Before → After | What Phase 0 did / what remains |
|---|---|---|
| PIN-TPL-008 | Open → **In Progress** | Characterization tests added. Host-testable firmware seams and CI remain (1C) |
| PIN-TPL-015 | Open → **In Progress** | `.TS` case fixed. `wifi_config` fallback, `C:/t`, unpinned Git patches, unused deps (D4 = no) remain |
| PIN-TPL-016 | Open → **In Progress** | Dead files removed, `remember()` de-duplicated, commented wiring relocated. Framing unification and the two remaining `TransportType` enums (firmware and Tauri) remain |
| PIN-TPL-018 | Open → **In Progress** | Pin map and strapping-pin warning documented in `board/mod.rs`. Pin change remains (needs approval) |
| PIN-TPL-020 | Open → **In Progress** | Module files split into schema/store/view. God-store redesign, `any` typing, `noImplicitAny` remain |
| PIN-TPL-021 | Open → **In Progress** | Internal misspellings fixed. Wire names stay frozen until a versioned change |
| PIN-TPL-022 | Open → **In Progress** | rustfmt clean. Clippy and compiler warnings (1 / 2 / 11) and CI enforcement remain |
| PIN-TPL-024 | Open → **In Progress** | README LiDAR statement and stale paths fixed. `.espConfig` / `pre-script.rhai` (frozen, C-10) and external-agent references remain |
| PIN-TPL-026 | Open (unchanged) | Ownership invariant documented in `core/hardware/console.rs`. The fix remains |
| All others | Open (unchanged) | Functional defects were deliberately not touched |

**New problems introduced:** none found. The checks above cover build, format, wire format, log helpers, contract strings and code moves.

## 8. What was not verified

- **No hardware or runtime test.** Nothing was flashed or connected, and the app was not launched in `tauri dev`. Equivalence rests on the static and build evidence in §5, not on observed device behavior.
- **Suggested smoke test**, for you to run or to authorize separately: `cd Firmware_Templates; cargo +esp-1.93 run --frozen --release` (flashes the board). Then `cd UI; bun run tauri dev`, connect over Bluetooth, and confirm three LED cards appear and each slider and toggle drives the right GPIO (12/14/27). Disconnect and reconnect to check replay.
- Firmware module logic has no host tests yet (PIN-TPL-008, Phase 1C).

## 9. Git status and recovery

- All work is on local branch `phase-0/structural-reorganization`: 8 commits on top of `5b5c8ed4`, plus this report's commit. The working tree is clean after commit. **Nothing has been pushed.**
- To review: `git log --oneline main..phase-0/structural-reorganization` and `git diff main...phase-0/structural-reorganization --color-moved=zebra`.
- To undo one step: `git revert <commit>`. To return to the original state: `git switch main`. `main` was never modified.
- To keep the result: merge or fast-forward `main` when you are satisfied (`main` has not moved, so a fast-forward is possible). Pushing is your decision.

## 10. Next step

Per the approval gates: **stop for review.** After you accept Phase 0, the next operation is the **post-Phase-0 reassessment and remediation roadmap**, which needs its own authorization.
