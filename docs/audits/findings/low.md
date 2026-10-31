# Low-severity findings

Part of the [issue register](README.md). Paths are repository-relative to the `Pinora_Templat` root, with line numbers at commit `5b5c8ed4`.

---

## PIN-TPL-021 — Misspelled identifiers: some are wire contracts, others are safe to fix

| Field | Value |
|---|---|
| Severity | Low |
| Affected files | Wire-visible (**must not change without a protocol version**): `lool_up_id` (`protocol/src/registration.rs:11`), `Ckick` (`protocol/src/module/buttonmodule.rs:5`), `Idol` (`protocol/src/module/lidar.rs:18`), `distant` (`protocol/src/module/lidar.rs:7`). Internal only (safe in Phase 0): `manuel_id` (field, `Firmware_Templates/src/core/modulecore.rs`), `TransportEmiter` / `transport_emiter.rs`, local `tansport` (`main.rs:69`), `buzer_pin`, `write_chuck` (`module/rfid.rs`), `testrang`, `UI/src/lib/ModuleGeter.ts`. Protocol-crate public type names `RangPoint`, `MddeRfid` do not appear on the wire but are public crate API, so they are frozen in Phase 0 (see contract C-08 in [compatibility contracts](../architecture/compatibility-contracts.md)) |
| Confidence | Confirmed (wire visibility checked by serialization probe) |
| Status | Open |
| Proposed phase | 0 (internal names) / 2A (wire names, versioned) |

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
| Confidence | Confirmed (see [validation baseline](../evidence/validation-baseline.md)) |
| Status | Open |
| Proposed phase | 0 |

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
| Confidence | Confirmed |
| Status | Open |
| Proposed phase | 3B |

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
| Confidence | Confirmed |
| Status | Open |
| Proposed phase | 0 (documentation only; `.espConfig` and `pre-script.rhai` content left unchanged; see contract C-10) |

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
| Confidence | Confirmed |
| Status | Open |
| Proposed phase | 1A (with PIN-TPL-004) |

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
| Confidence | Confirmed (static) |
| Status | Open |
| Proposed phase | 3B |

**What is wrong.** `configure_console_uart` calls `uart_driver_install` and `uart_vfs_dev_use_driver` on UART0 through `unsafe` `sys` calls, while `Peripherals::take()` still hands out `p.uart0` as if it were free.

**Why this is a problem.** Rust's peripheral-ownership guarantees no longer reflect reality. A future `UartDriver::new(p.uart0, …)` would fail at runtime or conflict.

**How it could affect Pinora.** A latent conflict if UART0 is ever used through the HAL. Today it is not.

**Recommended solution.** Document the invariant in Phase 0 with a `// SAFETY:`/ownership comment. Later, consume `p.uart0` explicitly so the type system enforces exclusivity.

**Risk of changing it.** Changes to console stdin buffering could affect the Serial transport.

**How we will verify the fix.** Build, plus Serial transport smoke test when authorized.
