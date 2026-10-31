# Phase 0 — Structural reorganization (PLAN)

| Field | Value |
|---|---|
| Phase | 0 |
| Status | **Awaiting Approval** |
| Baseline revision | `5b5c8ed4` (`main`, clean) |
| Author | Initial audit, 2026-10-31 |
| Linked findings | PIN-TPL-008 (characterization tests only), -015 (file-name case only), -016, -018 (documentation only), -020 (file organization only), -021 (internal names only), -022, -024 (README only), -026 (documentation only) |
| Behavioral impact | **None intended.** Two observable but non-functional differences are listed in §6 and need explicit acceptance |

## 1. Objective

Reorganize the template so a new developer or AI agent can find, understand and safely change each part, **without changing behavior**. Phase 0 also adds the *characterization tests* that later phases need to prove behavior is preserved.

Phase 0 does **not** fix any functional defect. Every High/Medium functional finding stays Open and is carried into the post-Phase-0 reassessment.

## 2. Frozen contracts

Everything in [compatibility-contracts.md](../../architecture/compatibility-contracts.md) (C-01 … C-19) is frozen. In particular:

- The whole `protocol/` public API and wire format (C-01, C-08). Phase 0 adds tests and doc comments there, and nothing else.
- Every manifest, lockfile, config file, `pinora.toml`, `.espConfig`, `pre-script.rhai` and `justfile` (C-10, C-11, C-17, C-19). Exceptions need the separate approvals in §8.
- Tauri IPC command and event names (C-06, C-07).
- Firmware startup order, hardware assignments, timing and error semantics (C-12 … C-16).

## 3. Proposed target layout

### 3.1 Firmware (`Firmware_Templates/src`)

```
main.rs                      entry: init → board::build_modules → runtime::run (same order as today)
board/
  mod.rs                     active wiring for this image (3 LEDs), moved verbatim from main.rs,
                             plus a documented pin map (GPIO12 strapping-pin warning, PIN-TPL-018)
  disabled_wiring.rs         the commented-out wiring for lidar/stepper/joystick/IR/rangefinder/IMU/RFID/servo,
                             moved verbatim (still comments; documented conflicts and hazards PIN-TPL-004/018)
runtime/
  mod.rs                     main loop moved verbatim (keeps `?` semantics, PIN-TPL-001 documented, not fixed)
core/
  mod.rs
  module.rs                  ← core/modulecore.rs  (Module trait, ModuleCore, ModuleError; lifecycle docs)
  hardware/
    mod.rs                   re-exports so existing `crate::core::hardware::*` users keep compiling
    pins.rs                  OutputPinCore, InputPinCore
    timing.rs                rtos_sleep_ms, now_us, TimerState
    context.rs               HardwareContext + shared-bus type aliases (non-Send invariant documented)
    system_info.rs           print_esp_system_info, format_bytes
    console.rs               configure_console_uart (moved from main.rs; UART0 ownership note, PIN-TPL-026)
  transport/
    mod.rs
    manager.rs               ← transport_core.rs (TransportCore, TransportType)
    emitter.rs               ← transport_emiter.rs (type renamed TransportEmiter → TransportEmitter)
    discovery_cache.rs       the identical `remember()` from wifi.rs and bluetooth.rs, extracted once
    serial.rs                serial_command_reader extracted from transport_core.rs
    bluetooth.rs             unchanged logic; worker state machine documented
    wifi.rs                  unchanged logic; blocking-boot and throughput limits documented
modules/                     ← module/
  mod.rs
  actuators/  led.rs ← ledmodule.rs · servo.rs ← servomodule.rs · stepper.rs
  sensors/    button.rs ← buttonmodule.rs · rangefinder.rs ← range_finder.rs · rfid.rs
              remote_receiver.rs · joystick.rs · imu/{mod.rs, types.rs ← imu_type.rs, mpu.rs ← mpu_impl.rs}
  composite/  lidar.rs
utilities/
  math.rs                    unchanged (overflow behavior documented, PIN-TPL-002)
```

The dead file `core/emitter.rs` is deleted only with approval D3 (§8). Otherwise it moves to `core/legacy_emitter.rs.txt` as an inert reference.

### 3.2 Protocol (`protocol/`)

There are no file moves and no renames (C-08). Phase 0 adds:
- `tests/wire_format.rs`: golden JSON for **every** command, event, registration and system variant, plus a deserialize round-trip. It uses only existing dependencies, so `Cargo.lock` is unaffected.
- `//!` and `///` doc comments that state each type's exact JSON shape and frozen misspellings (PIN-TPL-021).

### 3.3 Desktop backend (`UI/src-tauri/src`)

```
lib.rs                 run(): plugin setup, managed state, generate_handler! (same command list and order)
commands.rs            the 10 #[tauri::command] fns, names and signatures unchanged (C-06)
serial_transport.rs    AppState, connect(), shutdown(), COM reader thread (moved verbatim)
bluetooth_ports.rs     windows_bluetooth_ports() and the filter used by get_available_bluetooth_ports
wifi_transport.rs      unchanged logic
```

### 3.4 Desktop frontend (`UI/src`)

- `lib/Modulefront.TS` → `lib/Modulefront.ts`. This is a case-only rename with `git mv`, done in two steps on Windows. It fixes resolution on case-sensitive file systems (PIN-TPL-015) without changing behavior. Export names stay the same.
- Each `lib/Modules/<x>.tsx` is split into `lib/Modules/<x>/schema.ts`, `store.ts` and `view.tsx`, with an `index.ts` that re-exports the same names. Every importer keeps working through the index. The schema files contain no React. This covers all 9 families, including the inactive ones (PIN-TPL-020).
- Pure helpers move out of `Modulefront.ts` (for example `registrationHasParent` and the registration factory map) into `lib/registry/*.ts`. Call order and store shape are unchanged.
- `lib/ModuleGeter.ts` is deleted only with approval D3. Otherwise it is renamed to `lib/module-getter.ts` (it has no importers).
- `bun test` characterization tests for `lib/logs.ts` (`describeLog`/`matchesLog`) record **current** behavior, including the three throwing cases from PIN-TPL-006, marked as "current behavior, see PIN-TPL-006". Bun's built-in runner adds no dependency, so `bun.lock` is unaffected.

### 3.5 Documentation

- Root `README.md`: correct the stale LiDAR sentence (PIN-TPL-024), point to `docs/audits/`, and update the repository-structure table to the new paths.
- `Firmware_Templates/*.md`, `UI/README.md`: update file paths only.
- Doc comments where they explain *why*: module lifecycle, the reliable/lossy emission rule, queue capacities, the Bluetooth worker states, the non-`Send` hardware invariant, `unsafe` FFI sites (`// SAFETY:`), and known hazards with their finding IDs. No comments on trivial lines.

## 4. Implementation strategy

Each step is reviewable alone and leaves the tree building.

| Step | Content | Rule |
|---|---|---|
| P0.0 | Baseline capture (§5) and recovery checkpoint (§7) | No source edits |
| P0.1 | Characterization tests (protocol golden JSON; `bun test` for logs) | Tests only; must pass on the *unchanged* code |
| P0.2 | `cargo fmt` on all three Rust packages | Formatting only; separate step so moves stay readable in diffs |
| P0.3 | Firmware reorganization (§3.1) | Code moved **verbatim**; only `use`/`mod` paths and the internal renames listed in contract notes change |
| P0.4 | Tauri backend split (§3.3) | Verbatim moves; `generate_handler!` list unchanged |
| P0.5 | Frontend reorganization (§3.4) | Verbatim moves; public exports unchanged |
| P0.6 | Doc comments and documentation (§3.5) | Comments and docs only |
| P0.7 | Approved deletions (D3) and approved manifest tidy-ups (D4) | Only if approved |
| P0.8 | Full verification, `RESULT.md`, register update, mirror sync | — |

Internal renames, all private to their crate: `TransportEmiter`→`TransportEmitter`, `transport_emiter`→`emitter`, `manuel_id`→`lookup_name` (Rust field only; the wire field `lool_up_id` is unchanged), `tansport`→`transport`, `buzer_pin`→`buzzer_pin`, `write_chuck`→`write_chunk`, `testrang`→`self_test_angles`, `Ledmodule`→`LedModule`, `Buttonmodule`→`ButtonModule`. Protocol-crate names are **not** renamed.

## 5. Verification strategy

| # | Evidence | Pass criterion |
|---|---|---|
| V1 | Protocol golden JSON tests | Written and passing on baseline (P0.1); **byte-identical** results after every later step |
| V2 | `cargo +stable check`, `clippy` (protocol, Tauri) | Pass; warning count ≤ baseline |
| V3 | `cargo +esp-1.93 build --frozen` debug **and** `--release` | Pass on baseline and after P0.3. Release ELF size difference reported (expected small: panic-location and log-target strings, §6) |
| V4 | `tsc --noEmit`; Vite build to a scratch `--outDir` | Pass; bundle module count and size reported before and after |
| V5 | `bun test` characterization | Identical results before and after |
| V6 | Contract grep (C-06, C-07, C-03, C-04 strings) | Same occurrences in the same semantic places |
| V7 | `git diff --color-moved=zebra --color-moved-ws=allow-indentation-change` review of P0.3–P0.5 | Changed lines are limited to `mod`/`use` paths, the listed renames and comments |
| V8 | `main.rs`/`runtime` startup order review | Statement order identical to baseline `main.rs:58-295` (C-15) |
| V9 | No diff to manifests, lockfiles, configs, `pinora.toml`, `.espConfig`, `pre-script.rhai`, `justfile` | Empty diff, unless D4 is approved |
| V10 | `cargo fmt --check` | Clean |

Only hardware-independent checks are run. **No flashing and no device connection.** Phase 0 therefore cannot *prove* runtime equivalence on hardware. The result report will say so, and will list an optional smoke test (flash, connect over Bluetooth, toggle each LED) for you to run or to authorize separately.

Pre-existing failures (formatting, warnings) are reported separately from anything new.

## 6. Observable differences that Phase 0 would introduce (need acceptance)

1. **Firmware log targets.** `EspLogger` prints the Rust module path. Lines such as `Firmware_Templates::core::transport::transport_core` would become `Firmware_Templates::core::transport::manager`. The desktop ignores non-JSON lines, but anything parsing UART log prefixes would see the new names.
2. **Panic location strings** (file and line) embedded in the binary change, because code moves.

Both are cosmetic. If either is unacceptable, the alternative is to keep the module paths (`transport_core.rs`, `module/`), which limits the reorganization.

## 7. Recovery plan

- P0.0 records the baseline commit (`5b5c8ed4`) and confirms a clean tree. If the tree is not clean at start, work stops and you are asked first.
- With approval D1, work happens on a new local branch `phase-0/structural-reorganization` created from `main`. `main` is never modified.
- With approval D2, each step P0.1–P0.7 is one local commit, so any step can be reverted alone (`git revert <sha>`). **Nothing is pushed.**
- Full rollback: `git switch main`. The branch can be deleted later with your approval.
- No history rewriting, no force operations, no changes to other branches or repositories.

## 8. Decisions requested with the Phase 0 authorization

| ID | Decision | Recommendation |
|---|---|---|
| D1 | Create local branch `phase-0/structural-reorganization` | Yes |
| D2 | Make one local commit per step (no push) | Yes; this is what makes the checkpoints recoverable |
| D3 | Delete dead files `Firmware_Templates/src/core/emitter.rs` and `UI/src/lib/ModuleGeter.ts` (both unreferenced; Git history keeps them) | Yes |
| D4 | Allow manifest tidy-ups: remove unused `rand` and `embassy-time` from `Firmware_Templates/Cargo.toml` (this changes `Cargo.lock`), and pin `rev` on the three ESP Git patches to the commits already locked | No; defer to Phase 2C so Phase 0 never touches lockfiles |
| D5 | Use the existing firmware target directory `C:/t` for baseline and verification builds (writes build cache only) | Yes; without it, V3 cannot run |
| D6 | Accept the two cosmetic differences in §6 | Yes |

## 9. Out of scope (deferred, with destination)

All functional fixes: PIN-TPL-001 … -007, -009 … -014, -017, -019, -023, -025, plus the behavioral parts of -015, -016 (framing unification), -018 (pin changes) and -020 (store redesign). They are carried into the post-Phase-0 reassessment and roadmap.

## 10. Completion criteria

- V1–V10 pass, or each failure is explained and classified as pre-existing or new.
- `phases/phase-0/RESULT.md` is written. It covers original and new organization, renamed files, moved modules, extracted helpers, documentation added, tests and results, compatibility concerns, deferred problems, and Git status with recovery information.
- The issue register is updated (status changes only, history kept).
- The README, CHANGELOG and phase index are updated, then mirrored to `C:\Dev\Pinora\Audits\Pinora_Templat\` with SHA-256 verification.
- Then work **stops for review**.
