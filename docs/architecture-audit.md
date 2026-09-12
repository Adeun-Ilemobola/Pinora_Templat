# Pinora v0.5.0-alpha architecture audit

The [root README](../README.md) is the developer entry point and states the alpha policy. This review describes the working tree inspected on 2026-09-12, including pre-existing UI work. It records findings rather than implementing an architecture redesign. Source integration does not establish hardware reliability.

## Scope and ownership

The audit traced React routes and views, Zustand connection and module stores, Tauri commands and serial thread lifetime, firmware boot/registry/emitter, Rust wire types, all declared module families, manifests, configuration, scripts, tests, and legacy documentation. There is no root Cargo workspace, current Slint implementation, first-party CLI source, or repository CI workflow in the inspected file inventory.

Firmware owns live hardware state and UUIDs. The frontend owns its device-state projection and user interaction drafts. Tauri owns transport resources only. Rust protocol depends on Serde/serde_json; firmware depends on it through a path dependency. Desktop Rust has no protocol dependency. TypeScript schemas are handwritten and incomplete. There is no Rust crate dependency cycle in these first-party manifests, but frontend module/store imports form cycles.

## Architecture quality review

| Finding | Evidence | Practical consequence / alpha boundary |
|---|---|---|
| Desktop architecture changed from Slint to React/Tauri | [App](../UI/src/App.tsx), [backend](../UI/src-tauri/src/lib.rs), [manifest](../UI/package.json) | Old root documentation was materially misleading; replaced in this pass. Slint comments survive in justfile, not an active Slint subsystem. |
| Native bridge is separated from module semantics | [backend](../UI/src-tauri/src/lib.rs) forwards raw lines and strings | Useful resource/domain boundary, but validation occurs neither there nor robustly at frontend ingress. |
| Incomplete duplicated protocol | [TS command](../UI/src/lib/protocol/command.ts), [TS event](../UI/src/lib/protocol/event.ts), [Rust command](../protocol/src/command.rs) | TS aggregate unions omit Servo and multiple Rust families. LiDAR empty-struct command variants are TS string literals. No generated synchronization mechanism found. |
| Schema definitions are not enforced | [Modulefront](../UI/src/lib/Modulefront.TS), [IncomingCommand](../UI/src/lib/IncomingCommand.ts) | Receive path parses JSON and casts; outgoing command payload is `any`. Module dispatch casts without checking registered kind. Invalid shapes can pass into state handlers. |
| Store, schema and view coupling | [LED](../UI/src/lib/Modules/Led.tsx), [Servo](../UI/src/lib/Modules/servo.tsx), [Modulefront](../UI/src/lib/Modulefront.TS) | Modulefront imports factories while module files import Modulefront; protocol schema files import React-bearing module files. This complicates independent validation/testing and future transport replacement. |
| Reconnect state is retained | [Disconnect / RegistrationEvent](../UI/src/lib/Modulefront.TS) | Registry and lookup maps survive disconnect. Firmware uses new boot UUIDs, so reconnect can accumulate stale entries; lookup names overwrite independently. No session generation or replay contract. |
| Initial events precede registration | [servo constructor](../Firmware_Templates/src/module/servomodule.rs), [boot sequence](../Firmware_Templates/src/main.rs) | Servo setters emit before all modules register; frontend logs but cannot apply events to absent stores. Default servo registration does not replay its full state. LEDs also register without brightness replay. |
| Hierarchy does not imply routing | [LiDAR](../Firmware_Templates/src/module/lidar.rs), [main](../Firmware_Templates/src/main.rs) | Children register with parent IDs, but only top-level registry IDs receive generic commands. LiDAR has its own child motor command route; frontend ignores parent metadata. |
| Reader failures do not become connection transitions | [serial reader](../UI/src-tauri/src/lib.rs) | Timeout, EOF and real read errors all become empty data. Persistent errors may loop while frontend remains Connected. No maximum line buffer length. |
| Disconnected sends report success | [send_data](../UI/src-tauri/src/lib.rs) | With no port, returns `Ok(())`; UI cannot infer delivery. Module actions do not await/catch the command promise. Connect failures expose only the coarse `Error` enum through IPC. |
| Bounded events, unbounded ingress/history | [emitter](../Firmware_Templates/src/core/emitter.rs), [main](../Firmware_Templates/src/main.rs), [store](../UI/src/lib/Modulefront.TS) | Runtime output drops at capacity 128; command channel and frontend log history have no bound. Virtualized rendering does not bound memory. No ack/retry/backpressure contract spanning the system. |
| Error propagation is inconsistent | [main](../Firmware_Templates/src/main.rs), [servo](../Firmware_Templates/src/module/servomodule.rs), [LiDAR](../Firmware_Templates/src/module/lidar.rs) | Tick errors discarded, command errors exit main, offset writes unwrap, selected child writes ignore errors. Error recovery and severity are not unified. |
| Hardcoded startup has side effects | [HardwareContext](../Firmware_Templates/src/core/hardware.rs), [servo](../Firmware_Templates/src/module/servomodule.rs) | PCA9685 initialized regardless of requested UI use; servo writes a test sequence during construction. No runtime hardware configuration loader. |
| LiDAR remains experimental | [firmware](../Firmware_Templates/src/module/lidar.rs), [UI](../UI/src/lib/Modules/lidar.tsx), [page](../UI/src/page/LidarPage.tsx) | Firmware is compiled but inactive; frontend creates a fake module, ignores event state and has no-op command actions. Scan timer gate is commented out, step validation absent, and termination depends on exact endpoint equality. Chunk delivery can drop without replay. |
| UI coverage depends on unrelated LED presence | [Dashboard](../UI/src/page/Dashboard.tsx) | Servo cards are inside `connected && leds.length`; servo-only registration does not display them. |
| Inconsistent names and wire tagging | [registration](../protocol/src/registration.rs), [IMU](../protocol/src/module/imu/imu_type.rs), [LED](../protocol/src/module/ledmodule.rs) | `lool_up_id`, `manuel_id`, `Ckick`, `Idol`, and `RangPoint` coexist with corrected frontend names. IMU still uses internal tags while LED uses external tags; consumers must follow each actual type. |
| Tests have stale expectations | [event_package.rs](../protocol/tests/event_package.rs) | LED, Button and Rangefinder assertions still expect `event_type`; actual serialization uses external tags. No current frontend/backend integration test suite was found. |
| Tooling and template remnants | [justfile](../justfile), [legacy config](../.espConfig/esp_config.json), [Rhai script](../Firmware_Templates/pre-script.rhai) | Cargo UI recipes use the old directory layout; machine-specific legacy commands are not current instructions. No generator configuration/invocation was found to substantiate turnkey scaffolding. |
| Packaging/security remains template-level | [Tauri config](../UI/src-tauri/tauri.conf.json), [backend manifest](../UI/src-tauri/Cargo.toml) | Product name `ui`, identifier `com.owner.ui`, placeholder authors/description, `greet` example and `csp: null` remain. No production hardening is claimed. |
| Reproducibility differs by package | [ignore file](../.gitignore), [firmware manifest](../Firmware_Templates/Cargo.toml) | Firmware/Tauri/Bun locks tracked; protocol lock ignored. ESP Git patches have no `rev`; lockfile must preserve exact resolution. Bun/Node versions are not pinned by an engines/packageManager field. |
| Licensing is unspecified in current tree | File/manifest inventory | No license file or first-party license declaration found; historical MIT claims cannot be retained as current facts. |

These are areas for coordinated future work, not permission to rewrite them in this task. Their significance is architectural instability, not a claim that the author's current workflow is broken.

## Version inventory and decisions

All first-party packages in this tree are aligned to the same alpha release for this pass. Package SemVer is release metadata, not an independent protocol negotiation or compatibility number.

| File / declaration | Before | Meaning | Decision / reason |
|---|---|---|---|
| `pinora.toml` project.version | `0.1.0` | Pinora product/template release | `0.5.0-alpha` |
| `Firmware_Templates/Cargo.toml` package.version | `0.5.0` | Firmware first-party release | `0.5.0-alpha` |
| `protocol/Cargo.toml` package.version | `0.1.0` | First-party protocol crate package | `0.5.0-alpha`; align package release, without changing wire types |
| `UI/package.json` version | `0.1.0` | Private frontend package release | `0.5.0-alpha`; preserve existing dependency edits |
| `UI/src-tauri/Cargo.toml` package.version | `0.1.0` | Desktop native package release | `0.5.0-alpha` |
| `UI/src-tauri/tauri.conf.json` version | `0.1.0` | Application/bundle release | `0.5.0-alpha` |
| `Firmware_Templates/Cargo.lock` local Firmware_Templates entry | `0.5.0` | Local package metadata | `0.5.0-alpha` only |
| Same lock, local pinora-protocol entry | `0.1.0` | Local package metadata | `0.5.0-alpha` only |
| `UI/src-tauri/Cargo.lock` local ui entry | `0.1.0` | Local package metadata | `0.5.0-alpha` only |
| Existing ignored `protocol/Cargo.lock` local pinora-protocol entry | `0.1.0` | Local package metadata | `0.5.0-alpha` only; not regenerated or staged |
| Root/UI README | Historical/pre-alpha/template descriptions | Display/documentation | `v0.5.0-alpha` with current architecture and policy |
| `UI/bun.lock` | No root package version field | Dependency resolution | Unchanged; no field invented |
| `pinora.toml` schema_version | `1` | Metadata format | Unchanged; separate semantics |
| Cargo lock format / Bun lockfileVersion / configVersion | Existing values | Lockfile formats | Unchanged |
| Rust editions / rust-version / esp-1.93 | Existing values | Language/compiler constraints | Unchanged |
| ESP-IDF `v5.5.3`, Git source revisions | Existing values | SDK/dependency identity | Unchanged |
| All third-party package requirements/resolutions | Existing values | Dependencies | Unchanged |
| MFRC522 version register, IDF system-info field | Hardware/runtime values | Not Pinora release labels | Unchanged |
| Tauri schema URL `/config/2`, identifiers and UUIDs | Existing values | Config format/application identity | Unchanged |

No ambiguous first-party release number requiring a speculative bump was found. Separately meaningful schema/toolchain/hardware versions were deliberately excluded. No `cargo update`, broad upgrade, or lockfile regeneration was used. Four precise local package entries across three existing lockfiles were edited; all other lockfile bytes were preserved.

## Validation and boundaries

- Repository snapshot taken before edits, including untracked source and the existing ignored protocol lockfile. Existing UI work is preserved; `UI/package.json` receives only the additional release-version edit.
- JSON/TOML parsing and semantic comparisons check manifest dependencies and all non-version metadata. Lockfile comparisons permit only the identified local package versions; package dependency edges, registry checksums, Git revisions, and third-party versions stay identical.
- Root/UI/audit Markdown local links, section anchors, balanced fences, and diagram node/edge structure checked. Mermaid uses a basic flowchart with quoted labels; no browser-rendered diagram test was performed.
- Host `cargo +stable test --frozen --target-dir <temporary audit directory>/protocol-target`: compiled successfully; **6 passed, 3 failed**. Failing tests are `led_package_preserves_internal_tag`, `button_package_preserves_empty_struct_variant`, and `standalone_event_families_have_no_routing_ids` (Rangefinder assertion). Failures reflect existing unchanged source/test disagreement. Unused `PivotPoint` warning remains.
- `bun run tsc --noEmit`: **failed with 36 existing unused-import/local/parameter diagnostics** in AppLayout, LiDAR and Servo files. No output emission or dependency changes. These files were not edited by this task. Full UI packaging was not attempted because its configured TypeScript stage already fails.
- No new firmware compilation, hardware run/flash, toolchain installation or default switch. Firmware/protocol implementation source and compiler/IDF/optimization configuration remain byte-identical to the start of this task. ESP 1.93 remains selected. ESP 1.97 remains unsupported by project decision.
- Markdown files use LF line endings. Task-scoped diff whitespace checks pass when existing CRLF manifest endings are treated as line endings; unrelated pre-existing whitespace remains untouched.
- No stage/commit. Preserved compiler controls/trial artifacts `C:\b193`, `C:\c193`, `C:\c197` were not changed. This documentation pass does not replace their historical build evidence.

## Change attribution

Task changes: root README, this audit, UI README; six first-party release declarations; matching local package entries in firmware/Tauri lockfiles and the ignored standalone protocol lockfile. No application behavior changed.

Pre-existing changes: UI Bun lock and component configuration, frontend package dependencies, App/router, layout and reusable UI components, styles, Modulefront, LiDAR source, plus untracked firmware `.cursorignore` and `UI/src/page/LidarPage.tsx`. All retained. Their presence matters: a clean checkout without those uncommitted changes may not match this working-tree architecture snapshot.

## Verification limits

No fresh-clone setup, native packaging, serial device behavior, sensor accuracy, actuator timing, or cross-platform execution was verified in this audit. “Stable enough for current internal development” is the maintainer's project status; it is not inferred from the failing lightweight checks. Alpha architecture and external contracts remain explicitly unstable. No future feature or release-date commitment is inferred from placeholder code.
