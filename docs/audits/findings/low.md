# Low-severity findings

Part of the [issue register](README.md). Paths are repository-relative to the `Pinora_Templat` root, with line numbers at commit `5b5c8ed4`.

---

## PIN-TPL-021 — Misspelled identifiers: some are wire contracts, others are safe to fix

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | Wire-visible (**must not change without a protocol version**): `lool_up_id` (`protocol/src/registration.rs:11`), `Ckick` (`protocol/src/module/buttonmodule.rs:5`), `Idol` (`protocol/src/module/lidar.rs:18`), `distant` (`protocol/src/module/lidar.rs:7`). Internal only (safe in Phase 0): `manuel_id` (field, `Firmware_Templates/src/core/modulecore.rs`), `TransportEmiter` / `transport_emiter.rs`, local `tansport` (`main.rs:69`), `buzer_pin`, `write_chuck` (`module/rfid.rs`), `testrang`, `UI/src/lib/ModuleGeter.ts`. Protocol-crate public type names `RangPoint`, `MddeRfid` do not appear on the wire but are public crate API, so they are frozen in Phase 0 (see contract C-08 in [compatibility contracts](../architecture/compatibility-contracts.md)) |
| Current location (after Phase 0) | Wire names unchanged in `protocol/src/**` (frozen). Internal misspellings were fixed in Phase 0 |
| Confidence | Confirmed (wire visibility checked by serialization probe) |
| Status | **Blocked** (2026-11-02) on a major protocol version: internal names fixed (Phase 0), corrected spellings accepted as input (2A); renaming the output names is wire-breaking. See phases/phase-3/phase-3a/RESULT.md |
| Proposed phase | 2A (roadmap, pending approval) |

**What is wrong.** Spelling errors appear across identifiers. Some are serialized JSON keys or enum tags. Others are only Rust or TypeScript symbols.

**Why this is a problem.** They make the code harder to search and read. Without this list, someone could "fix" a wire name and break the desktop or other tools.

**How it could affect Pinora.** Renaming a wire name breaks compatibility; renaming an internal one improves readability at no risk.

**Recommended solution.** Phase 0 renames internal symbols only. Wire names are documented as frozen; correcting them is a versioned protocol change, with optional `#[serde(alias)]` for transition.

**Risk of changing it.** None for the internal set if the compiler passes. High for the wire set.

**How we will verify the fix.** Build, plus the Phase 0 golden-JSON characterization tests showing unchanged output.

---

## PIN-TPL-022 — Formatting and lint rules are not enforced

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | All three Rust packages: `cargo fmt --check` reports 78 diff hunks in `Firmware_Templates`, 21 in `protocol` and 6 in `UI/src-tauri`. Clippy: one warning in `protocol` (`Axes::add` shadows `std::ops::Add::add`) and two in `UI/src-tauri` (unused `ConnectionState` fields, unused `ConnectionType::Disconnected`). No ESLint or Prettier configuration in `UI` |
| Current location (after Phase 0) | rustfmt clean. Remaining: clippy (protocol 1, Tauri 2), firmware compiler warnings (11), no TypeScript lint/format config, no enforcement |
| Confidence | Confirmed (see [validation baseline](../evidence/validation-baseline.md)) |
| Status | **Resolved** in Phase 3A (2026-11-02): zero warnings in every build; every clippy step in CI denies warnings; rustfmt clean. TypeScript is gated by strict `tsc` (no separate linter: new dependency). See phases/phase-3/phase-3a/RESULT.md |
| Proposed phase | 3A (roadmap, pending approval) |

**What is wrong.** Code style is inconsistent (indentation, spacing, import grouping). Lints are not run.

**Why this is a problem.** Diffs are noisy, and real issues hide among cosmetic ones.

**How it could affect Pinora.** Review quality drops, especially for AI or contributor patches.

**Recommended solution.** Run `cargo fmt` (behavior-neutral) in Phase 0, fix the trivial clippy findings that do not change behavior, and add a formatter for TypeScript.

**Risk of changing it.** None functionally. `Axes::add` is public protocol-crate API, so renaming it waits until external consumers are checked.

**How we will verify the fix.** `cargo fmt --check` and `cargo clippy` are clean, and builds are unchanged.

---

## PIN-TPL-023 — Desktop packaging still carries Tauri scaffold defaults and leftovers

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | `UI/src-tauri/tauri.conf.json:3`, `:5`, `:21` (`productName: "ui"`, `identifier: "com.owner.ui"`, `csp: null`), window title `"ui"`; `UI/src-tauri/Cargo.toml` (`description = "A Tauri App"`, `authors = ["you"]`); `UI/src-tauri/src/lib.rs:46` (`greet` command), `:27-31` (unused `ConnectionState` fields), `:312-342` (`start_port` maps every error to `ConnectionType::Error`, losing the message) |
| Current location (after Phase 0) | `UI/src-tauri/tauri.conf.json:3`, `:5`, `:21`; `UI/src-tauri/src/commands.rs:17` (`greet`), `:171`; `UI/src-tauri/src/serial_transport.rs:29-33` (unused `ConnectionState` fields) |
| Confidence | Confirmed |
| Status | **Resolved** in Phase 3A (2026-11-02): CSP and devCsp, product name and title "Pinora", `greet` removed, `start_port` returns error detail. Identifier kept on purpose (app-data directory). CSP not checked in the running window. See phases/phase-3/phase-3a/RESULT.md |
| Proposed phase | 3A (roadmap, pending approval) |

**What is wrong.** Scaffold metadata was never customized. The Content Security Policy is disabled. One sample command is still exposed over IPC. `start_port` hides error detail, unlike `start_bluetooth_port`.

**Why this is a problem.** Packaged builds show as "ui". With CSP disabled, any script that ever gets injected (for example via a dependency) can call `send_data` and drive hardware. React's escaping currently prevents injection from device strings, and no `dangerouslySetInnerHTML` exists.

**How it could affect Pinora.** Confusing installs, and a weaker defense-in-depth posture for a hardware-control app.

**Recommended solution.** Set a real product name and identifier. Changing the identifier moves the app-data directory, so it needs approval and a migration note. Define a restrictive CSP. Remove `greet`. Return error detail from `start_port`.

**Risk of changing it.** The identifier change relocates user data such as theme storage. CSP needs testing against Vite dev mode.

**How we will verify the fix.** Packaged build inspection, plus a CSP violation check in the devtools console.

---

## PIN-TPL-024 — Documentation and legacy metadata have drifted from the code

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | `README.md:174` (says TypeScript LiDAR start/stop/test are strings; `UI/src/lib/Modules/lidar.tsx:75-77` now uses objects); `.espConfig/esp_config.json:6-7` (`source ~/export-esp.sh && cargo build`, `cargo flash … /dev/ttyUSB0`, project name `template`); `Firmware_Templates/pre-script.rhai` (lists ten chip targets; only classic ESP32 is configured); `README.md`, `docs/architecture-audit.md`, `Firmware_Templates/BLUETOOTH_TRANSPORT.md` refer to verification by a named external agent (“Codex”) whose records are not in the repository |
| Current location (after Phase 0) | `README.md` (states the LiDAR route inserts a `fake-lidar` module; no such code exists); `.espConfig/esp_config.json`; `Firmware_Templates/pre-script.rhai`; external-agent references in `README.md`, `docs/architecture-audit.md`, `Firmware_Templates/BLUETOOTH_TRANSPORT.md` |
| Confidence | Confirmed |
| Status | **Resolved** in Phase 3A (2026-11-02): README, architecture review and guides corrected. `.espConfig` and `pre-script.rhai` left untouched until the sibling audits (C-10). See phases/phase-3/phase-3a/RESULT.md |
| Proposed phase | 3A (roadmap, pending approval) |

**What is wrong.** Some statements are out of date, and legacy generator metadata contains commands that would not work on this project.

**Why this is a problem.** Readers, human or AI, act on stale guidance.

**How it could affect Pinora.** Wrong expectations about protocol shapes and setup commands.

**Recommended solution.** Correct the README in Phase 0. Annotate, but do not edit, `.espConfig` and `pre-script.rhai` until it is known whether `pinora_cli` or `pinora_studio` consume them (an external dependency that must not be audited now).

**Risk of changing it.** Editing generator metadata could break external tooling; only documentation is changed.

**How we will verify the fix.** Doc review against the code.

---

## PIN-TPL-025 — Some events report the requested value, not the applied one

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | `Firmware_Templates/src/module/servomodule.rs:157-169` (`set_min_pivot` / `set_max_pivot` clamp but emit the unclamped input); `Firmware_Templates/src/module/stepper.rs:311-322` (pivot limits emitted without validation). The LED case is covered by PIN-TPL-002 |
| Current location (after Phase 0) | `Firmware_Templates/src/modules/actuators/servo.rs:158-166`; `Firmware_Templates/src/modules/actuators/stepper.rs:317-328` |
| Confidence | Confirmed |
| Status | **Resolved** in Phase 1B (2026-10-31); Verified after the hardware check in phases/phase-1/phase-1b/RESULT.md (B-V6) |
| Proposed phase | 1B (roadmap, pending approval) |

**What is wrong.** Confirmation events echo the command payload instead of the state the module actually adopted.

**Why this is a problem.** The UI is told something that is not true.

**How it could affect Pinora.** Displayed limits differ from the real limits.

**Recommended solution.** Emit the stored, applied value.

**Risk of changing it.** Behavior change (event payload values).

**How we will verify the fix.** Unit test: a clamped input yields the clamped value in the event.

---

## PIN-TPL-026 — The console UART driver is installed with raw ESP-IDF calls, bypassing HAL ownership

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | `Firmware_Templates/src/main.rs:30-56`, `:64` |
| Current location (after Phase 0) | `Firmware_Templates/src/core/hardware/console.rs:9-37` (invariant now documented in the doc comment) |
| Confidence | Confirmed (static) |
| Status | **Resolved** in Phase 3A (2026-11-02): `configure_console_uart` consumes `p.uart0`. Verified on your board (`just hw-test`). See phases/phase-3/phase-3a/RESULT.md |
| Proposed phase | 3A (roadmap, pending approval) |

**What is wrong.** `configure_console_uart` calls `uart_driver_install` and `uart_vfs_dev_use_driver` on UART0 through `unsafe` `sys` calls, while `Peripherals::take()` still hands out `p.uart0` as if it were free.

**Why this is a problem.** Rust's peripheral-ownership guarantees no longer reflect reality. A future `UartDriver::new(p.uart0, …)` would fail at runtime or conflict.

**How it could affect Pinora.** A latent conflict if UART0 is ever used through the HAL. Today it is not.

**Recommended solution.** Document the invariant in Phase 0 with a `// SAFETY:`/ownership comment. Later, consume `p.uart0` explicitly so the type system enforces exclusivity.

**Risk of changing it.** Changes to console stdin buffering could affect the Serial transport.

**How we will verify the fix.** Build, plus Serial transport smoke test when authorized.

---

## PIN-TPL-016 — Duplicated logic and dead code obscure the real architecture

| Field | Value |
|---|---|
| Severity | **Low** (was Medium until the 2026-10-31 reassessment) |
| Affected files | `Firmware_Templates/src/core/emitter.rs` (not compiled: `core/mod.rs:3` comments it out); `TransportType` defined three times (`core/emitter.rs:9`, `core/transport/transport_core.rs:21`, `UI/src-tauri/src/lib.rs:40-44`); `remember()` duplicated (`wifi.rs:225`, `bluetooth.rs:330`); newline framing implemented four times (`wifi.rs:182-209`, `bluetooth.rs:238-262`, `UI/src-tauri/src/lib.rs:156-164`, `UI/src-tauri/src/wifi_transport.rs:223-244`); ~130 lines of commented-out wiring (`Firmware_Templates/src/main.rs:96-207`); `UI/src/lib/ModuleGeter.ts` (no importers); `transport_core.rs:121-125` (returns a receiver whose sender is dropped when no transport supplies one) |
| Current location (after Phase 0) | `TransportType` in `Firmware_Templates/src/core/transport/manager.rs:27` and `UI/src-tauri/src/serial_transport.rs:45`; framing in `Firmware_Templates/src/core/transport/wifi.rs`, `Firmware_Templates/src/core/transport/bluetooth.rs`, `UI/src-tauri/src/serial_transport.rs:170-178`, `UI/src-tauri/src/wifi_transport.rs:223-244`; dead-receiver fallback `Firmware_Templates/src/core/transport/manager.rs:126-130`. Dead files, duplicate `remember()` and commented wiring are resolved |
| Confidence | Confirmed |
| Status | **Resolved** in Phase 2B (2026-11-02): one `LineFramer` and limit table for every transport, and the dead-receiver fallback replaced by an error. See phases/phase-2/phase-2b/RESULT.md |
| Proposed phase | 2B (roadmap, pending approval) |

**What is wrong.** Several concepts exist in parallel copies, and dead files look live. Framing differs subtly: firmware Wi-Fi caps lines at 8 KiB, Tauri Wi-Fi at 256 KiB, and the Tauri COM reader is uncapped.

**Why this is a problem.** Readers cannot tell which implementation is authoritative. Fixes land in one copy only.

**How it could affect Pinora.** Higher maintenance cost and divergent behavior between transports.

**Recommended solution.** Phase 0: remove the uncompiled `core/emitter.rs` and the unused `ModuleGeter.ts` (each requires explicit approval as a deletion). Extract the identical `remember()` into a shared discovery-cache helper. Move commented-out wiring into a documented examples file. Phase 2: unify line framing with one tested helper per language, which changes limits and is therefore behavioral.

**Risk of changing it.** Low for the removals: the compiler proves those files are not referenced. Framing unification changes limits.

**How we will verify the fix.** `cargo check`, `tsc` and Vite build succeed. A grep confirms no remaining references.


**Reassessment note (2026-10-31).** Phase 0 removed the dead files, merged the duplicate `remember()` and moved the commented wiring. What remains is the duplicated newline framing with different size limits, plus the dead-receiver fallback. The framing limits are already covered by PIN-TPL-010 and PIN-TPL-007. The two remaining `TransportType` enums live in different crates and mean different things (firmware transport selection vs desktop COM-port kind), so they are not true duplication. The remaining impact is maintainability only, hence Low.

---

## PIN-TPL-027 — Module tick, registration and replay order changes on every boot

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | `Firmware_Templates/src/core/module.rs:37` (`ModuleRegistry = HashMap<String, …>`); `Firmware_Templates/src/runtime/mod.rs:20-24` (`register_all` iterates the map), `:44-46` (tick loop iterates the map) |
| Current location (after Phase 0) | Same as Affected files (found during the post-Phase-0 reassessment) |
| Confidence | Probable. `std::collections::HashMap` uses a randomly seeded `RandomState`, and on ESP-IDF the seed comes from the hardware RNG. Module UUIDs are also random. Not observed on hardware |
| Status | **Resolved** in Phase 2C (2026-11-02): insertion-ordered `ModuleRegistry`. **Verified on your board:** registration order led1, led2, led3 on three boots. See phases/phase-2/phase-2c/RESULT.md |
| Proposed phase | 2C (roadmap, pending approval) |

**What is wrong.** The registry is a `HashMap` keyed by random UUIDs, using randomized hashing. Iteration order, which drives `register()`/`first_emit()` order and the per-iteration `tick()` order, is therefore effectively different on every boot.

**Why this is a problem.** Bus access order between modules (for example several I²C devices), the order of registration messages, and therefore the order of replayed registrations all vary from boot to boot. Timing-dependent bugs become hard to reproduce. Construction order in `board/mod.rs` gives the impression that order is controlled, but it is not.

**How it could affect Pinora.** Today, with three LEDs, there is no visible effect. With several sensors on a shared bus, or with modules that depend on each other's tick order, behavior could differ between boots.

**Evidence.** `pub type ModuleRegistry<'a> = HashMap<String, ModuleHandle<'a>>;`, and `for (_id, module) in modules.iter_mut()` in `register_all` and the main loop.

**Recommended solution.** Keep modules in a `Vec` in construction order, with a separate `HashMap<String, usize>` index for command lookup, or use an insertion-ordered map. Document that construction order is tick order.

**Risk of changing it.** Behavior change: tick and registration order become fixed. This is low risk but must not happen in a structure-only phase.

**How we will verify the fix.** Host test: registration and tick order equal construction order across repeated runs.

---

## PIN-TPL-036 — Misleading sensor logs and values

| Field | Value |
|---|---|
| Severity | Low |
| Affected files (at `ca8eede`) | `modules/sensors/rangefinder.rs:64,123,194`; `modules/sensors/imu/mpu.rs` (`new`); `UI/src/components/Esp32StatsCard.tsx:14` |
| Confidence | Confirmed (static) |
| Status | **Resolved** 2026-11-05. Compiled. Not run on hardware: this module is not in the current image, and only the LEDs are attached. |
| Proposed phase | Floating-state audit (2026-11-05, after Phase 3A) |

**What is wrong.** A standalone rangefinder treated "not ready yet", which is normal between measurements, as a failure and logged Critical about once a second. `inter_measurement_ms: 60` was stored but never sent to the sensor. An MPU identification failure was logged as "Accel error". "Free heap" is a snapshot, shown as if live.

**Why this is a problem.** Logs and values claimed things that were not so.

**How it could affect Pinora.** A healthy sensor looks broken in the log.

**Evidence.** `Err(_) =>` in `Rangefinder::tick` also caught `SensorNotReady`.

**Fix.** Not-ready and invalid readings are no longer failures. The unused field is gone, without changing what the sensor does. The label and log text say what they mean.

**Risk of changing it.** None known.

**How it was verified.** Review and compile.
