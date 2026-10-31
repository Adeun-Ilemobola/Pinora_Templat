# Initial audit report — Pinora Template

| Field | Value |
|---|---|
| Project | `Pinora_Templat` (Pinora Template, v0.8.0-alpha) |
| Scope | `C:\Dev\Pinora\Pinora_Templat` only. Sibling projects were listed by name but not inspected |
| Audit date | 2026-10-31 |
| Baseline | `main` @ `5b5c8ed4`, clean working tree |
| Operation | Initial audit: discovery, assessment, documentation. **No source changes** |
| Hardware | None used. No flashing, connecting or radio activity |

## 1. Coverage

| Subsystem | Files inspected | Depth |
|---|---|---|
| Firmware core | `main.rs`, `core/modulecore.rs`, `core/hardware.rs`, `core/emitter.rs`, `core/transport/{transport_core,transport_emiter,bluetooth,wifi}.rs`, `utilities/math.rs` | Line-by-line |
| Firmware modules | `module/{ledmodule,buttonmodule,servomodule,stepper,lidar,range_finder,rfid,remote_receiver,joystick}.rs`, `module/imu/*` | Line-by-line |
| Firmware build | `Cargo.toml`, `Cargo.lock` (dependency revisions), `.cargo/config.toml`, `rust-toolchain.toml`, `build.rs`, `sdkconfig.defaults`, `pre-script.rhai` | Full |
| Protocol | all 17 files under `protocol/src`, `Cargo.toml` | Full |
| Desktop backend | `UI/src-tauri/src/{main,lib,wifi_transport}.rs`, `Cargo.toml`, `tauri.conf.json`, `capabilities/default.json`, `build.rs` | Line-by-line |
| Desktop frontend | `main.tsx`, `App.tsx`, `lib/Modulefront.TS`, `lib/IncomingCommand.ts`, `lib/ModuleGeter.ts`, `lib/logs.ts`, `lib/protocol/*`, `lib/Modules/{Led,button}.tsx` (full); `lib/Modules/{lidar,stepper,servo,rangefinder,rfid,imu,remote-receiver}.tsx` (schemas, commands, effects); `page/*`, `components/{TransportForm,LogViewer,Esp32StatsCard,ModuleCard,ParentControlledState,AppLayout}.tsx`; `package.json`, `tsconfig.json`, `vite.config.ts` | Full / targeted |
| Not inspected in depth | `UI/src/components/ui/*` (generated shadcn/Base UI primitives), `PivotSlider.tsx`, `StepperDial.tsx`, icons, `bun.lock` contents, UI styling | Out of scope (visual design) or generated |
| Documentation | `README.md`, `docs/architecture-audit.md`, `Firmware_Templates/{BLUETOOTH,WIFI}_TRANSPORT.md`, `UI/README.md`, `pinora.toml`, `.espConfig/esp_config.json`, `justfile`, `.gitignore` | Full |
| Dependency behavior | `esp-idf-sys` `start.rs` and `esp-idf-hal` `ledc.rs` at the locked revisions | Targeted |

## 2. Headline results

| Severity | Count |
|---|---|
| Critical | 0 (4 findings on an escalation watch-list) |
| High | 6 |
| Medium | 14 |
| Low | 6 |

### Most consequential findings

1. **PIN-TPL-001.** One failed command ends firmware command processing for good, with no restart and no report.
2. **PIN-TPL-003.** Bluetooth is discoverable at all times, auto-confirms pairing and has no application authorization. Any nearby device can control the board.
3. **PIN-TPL-002.** The LED brightness input is unchecked. Overflow reboots debug builds and silently mis-drives release builds (shown by probe).
4. **PIN-TPL-004 / -005.** The servo, stepper and LiDAR code waiting to be re-enabled moves hardware without a command and can run without bound. This is Critical-on-enable.
5. **PIN-TPL-006.** One malformed JSON line from the device can take down the desktop UI (shown by probe).
6. **PIN-TPL-007 / -008.** The cross-language contract is handwritten and partial, and there are no tests. Every later fix depends on fixing this.

### Architectural strengths to preserve

The shared Rust protocol crate. Peripheral ownership through HAL types and single-threaded `Rc` buses. Minimal-work ESP-IDF callbacks. Bounded firmware queues with explicit limits. The reliable/lossy emission split. Reconnect replay. The Wi-Fi path's two-way validation. Narrow, checked `unsafe` FFI. Per-module frontend stores and a virtualized log viewer. Candid existing documentation. Details are in [current-system.md §6](../architecture/current-system.md).

## 3. Method

1. Confirmed the repository boundary: the template is its own Git repository, and the parent is not.
2. Inventoried all 149 tracked files and the ignored local files (presence only; the secret file was not read).
3. Read every authored source file in scope. Classified each component as active, unverified, experimental, legacy, scaffolding or planned ([current-system.md §4](../architecture/current-system.md)).
4. Traced commands, events and connection lifecycles across firmware, Tauri and React ([communication-flow.md](../architecture/communication-flow.md)).
5. Ran hardware-independent static checks with build output isolated from the repository. Ran two probes against the real sources ([validation-baseline.md](../evidence/validation-baseline.md)).
6. Ranked findings by real impact on the active image. Code that is present but not instantiated was rated with an explicit escalation note instead of being inflated.
7. Listed every external contract that Phase 0 must freeze ([compatibility-contracts.md](../architecture/compatibility-contracts.md)).

## 4. Limitations

- **No runtime behavior was verified.** That covers firmware on hardware, Bluetooth pairing, Wi-Fi, and the desktop app in `tauri dev`. Statements about runtime effects carry a "Probable" confidence where they are inferred.
- **The firmware was not compiled in this audit** (it would write to the shared `C:/t` cache). Earlier builds were reported passing by previous documentation; this audit did not re-check that.
- The Vite production build was not run (it writes `UI/dist`). `tsc --noEmit` passed.
- External consumers of the template's layout and metadata (`pinora_cli`, `pinora_studio`) were not inspected, by instruction. Contract C-10 therefore treats them as unknown and frozen.
- Branches other than `main` were not audited.
- Board wiring, eFuse state and actual LED circuits are unknown (relevant to PIN-TPL-018).

## 5. Recommended next step

Authorize **Phase 0, Structural Reorganization**, as specified in [phases/phase-0/PLAN.md](../phases/phase-0/PLAN.md), together with decisions D1–D6. Phase 0 changes structure, names, comments and tests only. It freezes every contract in C-01 … C-19, and it stops for review when done.
