# Medium-severity findings

Part of the [issue register](README.md). Paths are repository-relative to the `Pinora_Templat` root, with line numbers at commit `5b5c8ed4`.

---

## PIN-TPL-007 — The Rust↔TypeScript protocol contract is handwritten, partial, unversioned and validated inconsistently

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `protocol/src/**` (authoritative Rust types); `UI/src/lib/protocol/*.ts`; module schemas inside `UI/src/lib/Modules/*.tsx`; `UI/src-tauri/src/lib.rs:249-275` (`send_data` writes unvalidated strings to COM); `UI/src-tauri/src/wifi_transport.rs:92-96`, `:233` (Wi-Fi validates both directions) |
| Current location (after Phase 0) | `protocol/src/**`; `UI/src/lib/protocol/*.ts`; `UI/src/lib/Modules/*/schema.ts`; `UI/src-tauri/src/commands.rs:95` (COM write, unvalidated); `UI/src-tauri/src/wifi_transport.rs:92-96`, `:233` |
| Confidence | Confirmed |
| Status | Open |
| Proposed phase | 2A (roadmap, pending approval) |

**What is wrong.** The wire format is defined once in Rust and again by hand in Zod. The TypeScript aggregate schemas cover only `Led`, `Button` and `SysLog` events and `Led` commands. Ten Rust event families and six command families exist. The receive path never calls the Zod schemas; it uses `JSON.parse` and `as` casts. Tagging is mixed: most enums are externally tagged, `ImuEvent` is internally tagged (`event_type`), and `RangefinderCommandPayload::StartRanging` serializes as a bare string, `{"Rangefinder":"StartRanging"}`, while sibling commands are objects. There is no protocol version or capability handshake. Validation depends on the transport: Wi-Fi validates in Tauri, while USB/SPP do not validate on the desktop at all.

**Why this is a problem.** Any change to a Rust type can silently desynchronize the desktop. The README already describes TypeScript LiDAR commands as strings when the code now uses objects, which is evidence that drift happens. Firmware and desktop from different revisions cannot detect incompatibility.

**How it could affect Pinora.** Adding a module or changing one field produces runtime casts that succeed on bad data, or messages that are silently ignored. It is also the root enabler of PIN-TPL-006.

**Evidence.** `UI/src/lib/protocol/event.ts:18-33` (three variants) vs `protocol/src/module_event.rs` (ten variants). Probe output: `Rangefinder StartRanging: {"Rangefinder":"StartRanging"}` and `IMU Mode event: {"Imu":{"event_type":"Mode","mode":"Idle"}}`.

**Recommended solution.** Generate TypeScript types and Zod schemas from the Rust crate, for example with `ts-rs`, `specta` or `schemars` → JSON Schema → Zod. Alternatively, keep the handwritten schemas but enforce them with shared golden JSON fixtures tested on both sides. Validate on every transport, preferably once in Tauri. Add a `protocol_version` to the system snapshot or handshake. Any change to tagging is a breaking wire change and must be versioned.

**Risk of changing it.** Generation tooling changes the build. Fixing the tag inconsistencies breaks wire compatibility and must not happen in Phase 0.

**How we will verify the fix.** Golden fixture tests: every Rust variant is serialized to JSON, and the same files parse under the TypeScript schemas in CI.

---

## PIN-TPL-008 — No automated tests and no CI anywhere in the template

| Field | Value |
|---|---|
| Severity | Medium (a prerequisite for safely fixing every other finding) |
| Affected files | Entire repository; no `#[test]`, `tests/`, `*.test.ts`, or CI configuration exists |
| Current location (after Phase 0) | Characterization tests now exist in `protocol/tests/` and `UI/tests/`. Missing: firmware host tests, CI, and test scripts in `package.json`/`justfile`; `UI/tests` is outside `tsc`'s `include` |
| Confidence | Confirmed (`git ls-files` and source search) |
| Status | In Progress (Phase 0: characterization tests; Phase 1A: firmware host harness, `just test`, CI workflow; see phases/phase-1/phase-1a/RESULT.md) |
| Proposed phase | 1A (roadmap, pending approval) |

**What is wrong.** There are no unit tests, contract tests, integration tests or CI workflow. Firmware logic is entangled with ESP-IDF types, so none of it can run on a host. Examples are `TimerState` calling `esp_timer_get_time` and modules owning `PinDriver`/`LedcDriver` directly.

**Why this is a problem.** Phase 0 cannot prove behavior preservation beyond "it compiles". Every remediation phase would otherwise rely on hardware testing, which this process restricts.

**How it could affect Pinora.** Regressions in wire format, scan logic, motion limits and parsing go unnoticed until hardware misbehaves.

**Recommended solution.** Phase 0: add *characterization* (golden) tests for the protocol crate's JSON output, plus TypeScript tests for `describeLog`/`matchesLog` that record current behavior, including the current throws, without fixing anything. Phase 1C: introduce a thin hardware-abstraction seam (`Clock`, `PwmChannel`, `OutputPin` traits) so module state machines run under `cargo test` on the host. Add a CI workflow covering `cargo fmt --check`, `clippy`, protocol and Tauri tests, `tsc` and frontend tests; firmware `check` is optional because of the toolchain size.

**Risk of changing it.** Low for the tests themselves. The abstraction seam touches every module and belongs in a later phase.

**How we will verify the fix.** The tests run green in CI and fail when a golden fixture is deliberately altered.

---

## PIN-TPL-009 — The firmware main loop busy-polls and offers no timing guarantees

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `Firmware_Templates/src/main.rs:272-295` (`:291` yields 1 ms only every ≥650 ms); `Firmware_Templates/src/core/hardware.rs:103-145` (`now_us`, `TimerState`); `Firmware_Templates/src/module/remote_receiver.rs:121-140` (IR decode by polling) |
| Current location (after Phase 0) | `Firmware_Templates/src/runtime/mod.rs:42-66` (yield rule `:62`); `Firmware_Templates/src/core/hardware/timing.rs`; `Firmware_Templates/src/modules/sensors/remote_receiver.rs:121-140` |
| Confidence | Probable. The loop structure is confirmed. Idle-task starvation and IR timing failure are inferred, not measured |
| Status | Open |
| Proposed phase | 2C (roadmap, pending approval) |

**What is wrong.** The loop spins continuously. It ticks every module, then processes at most one command. It sleeps 1 ms only once at least 650 ms have elapsed. Every timed behavior is driven by `TimerState` deadlines checked from that loop: stepper steps every 1.25 ms, RFID every 20 ms, LiDAR every 30 ms. The NEC IR decoder samples a GPIO level each iteration and needs sub-350 µs resolution.

**Why this is a problem.** The main task occupies its core for ~99.8 % of the time. That starves priority-0 work, including the FreeRTOS idle task, which frees deleted-task memory and feeds idle hooks. Timing depends on whatever else happens in the loop, so an I²C transaction or UART logging stalls every module. Command latency grows with module count. Power draw is maximal.

**How it could affect Pinora.** Inactive modules that depend on tight timing (IR receiver, stepper speed, LiDAR dwell) will behave inconsistently once several are active together.

**Recommended solution.** Define a scheduling model, then implement it in Phase 2 because it changes timing. Options: a fixed-rate tick with `vTaskDelayUntil`; blocking on the command channel with a timeout equal to the next deadline; or dedicated tasks or the RMT peripheral for timing-critical drivers (RMT is the ESP-IDF-native IR receiver).

**Risk of changing it.** This changes concurrency and timing behavior, which Phase 0 explicitly forbids. Every module's timing must be re-verified.

**How we will verify the fix.** Measure loop period and jitter with `esp_timer` instrumentation and log a histogram. Check that the idle task runs (FreeRTOS run-time stats). Run the IR decoder against a logic-analyzer capture when hardware testing is authorized.

---

## PIN-TPL-010 — Telemetry backpressure: three stacked queues, silent drops, no rate limiting

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `Firmware_Templates/src/core/transport/transport_emiter.rs:34`, `:45-62`; `Firmware_Templates/src/core/transport/transport_core.rs:61-100`; `Firmware_Templates/src/core/transport/bluetooth.rs:155-169`, `:315-328`; `Firmware_Templates/src/core/transport/wifi.rs:69-81`, `:173-182`; `Firmware_Templates/src/module/imu/mpu_impl.rs:198-220`; `Firmware_Templates/src/module/range_finder.rs:177-195`; `Firmware_Templates/src/module/lidar.rs:267-317` |
| Current location (after Phase 0) | `Firmware_Templates/src/core/transport/emitter.rs:38`, `:49-66`; `Firmware_Templates/src/core/transport/manager.rs`; `Firmware_Templates/src/core/transport/bluetooth.rs`; `Firmware_Templates/src/core/transport/wifi.rs:174-234` (`:139` timeout); `Firmware_Templates/src/modules/sensors/imu/mpu.rs:198-220`; `Firmware_Templates/src/modules/sensors/rangefinder.rs:175-195`; `Firmware_Templates/src/modules/composite/lidar.rs:209-333` |
| Confidence | Confirmed (structure). Wi-Fi throughput ceiling derived from code: ≤ 1 outbound frame per ≥ 50 ms read timeout when the desktop is silent |
| Status | Open |
| Proposed phase | 2B (roadmap, pending approval) |

**What is wrong.**
- Messages pass through three bounded stages of 128: `TransportEmiter` → transport queue → Bluetooth `pending`. Runtime events use `try_send` at each stage. Drops are logged per message with `log::warn!`, and that logging itself writes to the 115 200-baud UART.
- Nothing limits emission rate. The IMU emits on every tick whose raw reading changed, which for a noisy sensor means every tick. The rangefinder emits a `Critical` SysLog on every failed tick. LiDAR emits three SysLogs per scan step.
- The Wi-Fi client handles at most one outbound frame per loop iteration and blocks up to 50 ms in `read`. With no inbound traffic that caps Wi-Fi telemetry at about 20 messages per second.
- `emit_reliable` blocks the caller indefinitely when queues are full. It is only used at startup today.

**Why this is a problem.** High-rate sensors crowd out low-rate but important events such as LED brightness confirmations and errors. Under load, drop logging adds to UART pressure. Wi-Fi cannot carry LiDAR or IMU rates.

**How it could affect Pinora.** Desktop state silently diverges from hardware, because confirmation events can be dropped. Enabling one sensor degrades every other module's reporting.

**Recommended solution.** Introduce per-module emission policies (rate limit, coalesce latest value, priority classes) and a single egress queue with drop accounting reported periodically as one SysLog. Make the Wi-Fi worker non-blocking: separate reader and writer, or `select`-style polling. Consider a dropped-message counter in the system snapshot.

**Risk of changing it.** This changes the timing and ordering of telemetry, which are runtime semantics, so it is not Phase 0 work.

**How we will verify the fix.** Host-side tests of the policy logic. Bench measurement of delivered vs emitted counts under a synthetic flood, once a test harness exists.

---

## PIN-TPL-011 — Frontend log history is unbounded and copied on every message

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `UI/src/lib/Modulefront.TS:294` (`logs: [...state.logs, event]`), `:263` (`console.info` per message); `UI/src/main.tsx:8-39` (every console call forwarded over Tauri IPC); `UI/src/components/LogViewer.tsx:113-126` (re-filters full history on each update); `UI/src/lib/logs.ts:88` (`JSON.stringify` per entry when searching) |
| Current location (after Phase 0) | `UI/src/lib/Modulefront.ts:259`, `:290`; `UI/src/main.tsx:8-39`; `UI/src/components/LogViewer.tsx:113-126`; `UI/src/lib/logs.ts:88` |
| Confidence | Confirmed (static). The slowdown was not benchmarked |
| Status | Open |
| Proposed phase | 2D (roadmap, pending approval) |

**What is wrong.** Each incoming message copies the full `logs` array, which is O(n) per message and O(n²) over a session. It also triggers a full re-filter in `LogViewer`, which with a search term stringifies every entry. Every message is additionally logged with `console.info`, which `main.tsx` forwards through the Tauri log plugin, adding a second IPC round-trip per message. The forwarding promise is not awaited or caught.

**Why this is a problem.** Memory and CPU grow without bound during long sessions or high-rate telemetry. The UI slows, then becomes unresponsive. That matters most while hardware is active.

**How it could affect Pinora.** LiDAR, IMU or rangefinder streams make the desktop unusable within minutes. The rendering is virtualized, so the cost hides in state updates rather than in the DOM.

**Recommended solution.** Use a bounded ring buffer (e.g. the last 5 000 entries) with an export option. Batch state updates per animation frame. Precompute the searchable text once per entry. Remove per-message `console.info`, or gate it behind a debug flag.

**Risk of changing it.** History older than the bound becomes unavailable in the UI. Users may need an export option to keep it.

**How we will verify the fix.** Synthetic test pushing 100 000 messages: memory stays bounded and append cost is flat. A React profiler check of LogViewer commit times.

---

## PIN-TPL-012 — Connection state can desynchronize from reality, and discovery depends on timing

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `UI/src-tauri/src/lib.rs:144-150` (USB read error emits no event; only Bluetooth does); `Firmware_Templates/src/main.rs:280` (`"SSI"` rediscovery branch empty); `Firmware_Templates/src/core/transport/transport_core.rs:101-125` (Serial path: no registration cache or replay); `UI/src/lib/Modulefront.TS:116-137` (registry cleared *after* `start_port` resolves); `Firmware_Templates/src/core/modulecore.rs:57-59` (`first_emit` default is empty; LED does not report state) |
| Current location (after Phase 0) | `UI/src-tauri/src/serial_transport.rs:155-163`; `Firmware_Templates/src/runtime/mod.rs:51` (`"SSI"` no-op); `Firmware_Templates/src/core/transport/serial.rs` (no replay); `UI/src/lib/Modulefront.ts:110-133`; `Firmware_Templates/src/core/module.rs` (`first_emit` default) |
| Confidence | Confirmed (static) |
| Status | Open |
| Proposed phase | 2A (roadmap, pending approval) |

**What is wrong.**
1. When a USB Serial read fails, for example because the cable is unplugged, the reader thread logs and exits. The UI keeps showing "Connected" because only the Bluetooth path emits a disconnect event.
2. The Serial firmware transport does not cache or replay registrations. A desktop that connects after boot sees no modules unless the board reset. That only happens if the DTR/RTS toggle in `start_port` actually resets it. The `SSI` rediscovery command does nothing.
3. In `Connect` (Serial), the registry and logs are cleared in the `.then` after `start_port` returns. Registrations that arrive in between are lost.
4. After any (re)connection, the UI creates module stores with default state (LED `0`). The firmware never replays current values, so the dashboard can show "Off" for an LED that is lit.

**Why this is a problem.** The operator sees an interface that does not match the hardware. That is the worst kind of UI fault for a control tool.

**How it could affect Pinora.** Wrong LED state after reconnect today. With actuators, a wrong position display could lead to a dangerous follow-up command.

**Recommended solution.** Emit a connection-lost event for every transport. Implement rediscovery: a defined command that makes the firmware re-send its system snapshot, registrations, and current state through each module's `first_emit`. Clear frontend state *before* opening the port. Mark module state as "unknown until reported" instead of defaulting to 0.

**Risk of changing it.** Touches IPC event semantics and the wire protocol (rediscovery request), so this belongs in Phase 2 and must be versioned per PIN-TPL-007.

**How we will verify the fix.** Tauri unit test of the reader's error path. Frontend store tests for the connection sequence. Hardware: unplug and replug USB, reconnect Bluetooth, and confirm the UI shows the true LED state.

---

## PIN-TPL-013 — Wi-Fi transport: development-grade security and fragile startup

| Field | Value |
|---|---|
| Severity | Medium (Wi-Fi is not the selected transport in the current image) |
| Affected files | `Firmware_Templates/src/core/transport/wifi.rs:50-52` (blocking connect at boot), `:88-110` (plaintext token); `Firmware_Templates/src/main.rs:69` (`?` on transport creation); `UI/src-tauri/src/wifi_transport.rs:50` (bind `0.0.0.0`), `:134-164` (per-byte reads with 3 s timeout), `:166-216` (single-threaded accept); `protocol/src/lib.rs:6` (secrets module compiled into both binaries) |
| Current location (after Phase 0) | `Firmware_Templates/src/core/transport/wifi.rs:40-77` (blocking connect `:61-63`), `:99-121`; `Firmware_Templates/src/main.rs` (`TransportCore::new(...)?`); `UI/src-tauri/src/wifi_transport.rs:50`, `:134-164`, `:166-216` |
| Confidence | Confirmed (static). Embedding of secrets in the Bluetooth-selected firmware image is Probable, because the Wi-Fi code is compiled even when not selected |
| Status | Open |
| Proposed phase | 2B (roadmap, pending approval) |

**What is wrong.**
- The shared token is sent in cleartext and compared with a non-constant-time comparison. It is static per build and compiled into both binaries. The SSID and password are compiled into the firmware.
- The listener binds every interface. Accept, authentication and reading share one thread. Authentication reads one byte at a time with a 3 s timeout per read, so an unauthenticated peer trickling bytes can occupy the listener for up to ~25 minutes (512 bytes × 3 s).
- If the access point is unavailable at boot, `Wifi::new` fails and `main` returns before any module is created. There is no Wi-Fi reassociation after a drop; only TCP reconnects.

**Why this is a problem.** Anyone on the same LAN who learns the token from traffic or from either binary can drive the device. One stalled peer blocks the legitimate device. A router reboot can leave the board dead until power-cycled.

**How it could affect Pinora.** It is acceptable for bench development, which the README acknowledges, but unsafe for any shared network or for actuator control.

**Recommended solution.** Phase 2: TLS or a challenge-response handshake with per-device keys provisioned at runtime (NVS), not at compile time. Accept on one thread and authenticate on a worker with an overall deadline. Make Wi-Fi bring-up retry in the background so the rest of the firmware starts. Default the listener bind to a configurable interface.

**Risk of changing it.** Handshake changes are wire-breaking, and provisioning changes the setup workflow.

**How we will verify the fix.** Tauri unit tests of the authenticator with slow and malformed peers. Firmware boot test with no AP present, when authorized.

---

## PIN-TPL-014 — Tauri commands do blocking work on the UI-facing thread

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `UI/src-tauri/src/lib.rs:190-227` (spawns PowerShell CIM query), `:117` (100 ms sleep while holding the state mutex), `:77` (`join` on reader thread), `:229-246`, `:312-342` (synchronous commands) |
| Current location (after Phase 0) | `UI/src-tauri/src/bluetooth_ports.rs:8-27`; `UI/src-tauri/src/serial_transport.rs:121`; all commands in `UI/src-tauri/src/commands.rs` are synchronous |
| Confidence | Probable: Tauri 2 runs non-`async` commands on the main thread. UI stall not measured |
| Status | Open |
| Proposed phase | 2D (roadmap, pending approval) |

**What is wrong.** All commands are synchronous `fn`s. `get_available_bluetooth_ports` launches `powershell.exe` and waits for a CIM query, typically hundreds of milliseconds to seconds. `start_bluetooth_port` runs it again before connecting. `start_port` sleeps 100 ms while holding `Mutex<AppState>`, which `send_data` also needs.

**Why this is a problem.** The window can freeze during port refresh and connect. Commands queue behind the mutex.

**How it could affect Pinora.** The app is sluggish or unresponsive during connection management. Commands sent during a connect are delayed.

**Recommended solution.** Make the commands `async` or move the work to `spawn_blocking`. Cache Bluetooth port metadata. Do not sleep while holding locks.

**Risk of changing it.** IPC command names and argument shapes must not change. Making them `async` is invisible to the frontend.

**How we will verify the fix.** Time the main-thread stall with the Tauri devtools performance panel before and after.

---

## PIN-TPL-015 — Builds are machine-specific and not reproducible from a fresh clone

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `protocol/src/lib.rs:6` + `.gitignore` (`/protocol/src/wifi_config.rs` required but untracked); `Firmware_Templates/.cargo/config.toml:4` (`target-dir = "C:/t"`); `Firmware_Templates/Cargo.toml:17-20` (Git patches without `rev`); `.gitignore` (`/protocol/Cargo.lock` ignored); `UI/src/lib/Modulefront.TS` (upper-case `.TS` extension); `justfile` (`clean-ui` uses PowerShell syntax; `[windows]` shell only); `Firmware_Templates/Cargo.toml:29`, `:32` (`rand`, `embassy-time` declared but unused by source) |
| Current location (after Phase 0) | `protocol/src/lib.rs:22`; `Firmware_Templates/.cargo/config.toml:4`; `Firmware_Templates/Cargo.toml:17-20`, `:29`, `:32`; `.gitignore`. The `.TS` extension is fixed |
| Confidence | Confirmed, except the `.TS` case-sensitivity break, which is Probable: `tsc` on Windows resolved it as `Modulefront.ts`, and Linux or case-sensitive macOS would not |
| Status | In Progress (Phase 0 partial; see phases/phase-0/RESULT.md §7) |
| Proposed phase | 2E (roadmap, pending approval) |

**What is wrong.** A fresh clone fails every Rust build until the developer creates an untracked secrets file. The firmware target directory is a hard-coded Windows drive path. `cargo update` would pull the moving `master` of three ESP-IDF crates. The standalone protocol lockfile is not tracked. One TypeScript source file has an upper-case extension, so it resolves only on case-insensitive file systems.

**Why this is a problem.** Onboarding a new developer or AI agent, CI, and cross-platform builds all fail or produce different dependency graphs.

**How it could affect Pinora.** Hard-to-diagnose build failures. An unintended ESP-IDF crate upgrade could change hardware behavior.

**Recommended solution.** Provide a compile-time default or build-script fallback for `wifi_config`, for example `option_env!` or a feature-gated module. That is a behavior change, so it goes in Phase 2C. Pin `rev` on the Git patches to the commits already in `Cargo.lock`, which is behavior-neutral. Use a relative or environment-provided target dir. Rename `Modulefront.TS` → `Modulefront.ts` with `git mv`, a Phase 0 candidate. Drop unused dependencies (Phase 0 candidate, verified by build).

**Risk of changing it.** Pinning `rev` must match the lockfile exactly or the build changes. Changing `target-dir` invalidates existing caches; preserve `C:/t` unless approved.

**How we will verify the fix.** Clean-clone build in a temporary directory with only documented setup steps.

---

## PIN-TPL-016 — moved to Low

Re-rated **Medium → Low** in the post-Phase-0 reassessment (2026-10-31). The full record now lives in [low.md](low.md). Kept here as a pointer so the history is not lost.

---

## PIN-TPL-017 — Module identity and hierarchy are only partly implemented

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `Firmware_Templates/src/main.rs:277-287` (top-level UUID routing only); `Firmware_Templates/src/module/lidar.rs:183-189` (children registered), `:327-336` (child addressing via `ChangeMotorAngle { id }`); `Firmware_Templates/src/core/modulecore.rs:27` (`parent_id` → empty string); `Firmware_Templates/src/core/transport/transport_emiter.rs:67` (SysLog source `"log"`); `UI/src/lib/Modulefront.TS:332` (lookup-name map overwritten on duplicates) |
| Current location (after Phase 0) | `Firmware_Templates/src/runtime/mod.rs:49-58`; `Firmware_Templates/src/modules/composite/lidar.rs:193`, `:337`; `Firmware_Templates/src/core/transport/emitter.rs:72`; `UI/src/lib/Modulefront.ts:328` |
| Confidence | Confirmed |
| Status | Open |
| Proposed phase | 2A (roadmap, pending approval) |

**What is wrong.** Composite modules (LiDAR, joystick) register their children, but commands addressed to a child UUID are rejected as "No top-level module". The only child control is a LiDAR-specific command carrying the child ID. Lookup names are not checked for uniqueness on either side. `None` parent IDs are serialized as `""`. Emitter-level logs use the pseudo-ID `"log"`, which the frontend reports as "Module store not found".

**Why this is a problem.** There is no general rule for addressing a module. Every composite module invents its own.

**How it could affect Pinora.** Adding composite modules multiplies special cases. Duplicate lookup names silently hide one module in the UI.

**Recommended solution.** Define a module tree in the firmware registry (parent → children) with a routing rule. Enforce lookup-name uniqueness at registration. Reserve and document system IDs (`"SSI"`, `"log"`). Changing `""` to `null` is wire-breaking and needs versioning.

**Risk of changing it.** Wire and registry semantics change, so it is not Phase 0 work.

**How we will verify the fix.** Registry unit tests (host) for routing and duplicate rejection.

---

## PIN-TPL-018 — Pin assignments are hard-coded in `main.rs`; one active LED uses a boot-strapping pin; latent conflicts in commented wiring

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `Firmware_Templates/src/main.rs:209-254` (LEDs on GPIO 12/14/27); commented alternates reuse the same pins: `:130` joystick button GPIO12, `:172-174` RFID RGB on GPIO12/14/27 |
| Current location (after Phase 0) | `Firmware_Templates/src/board/mod.rs` (pin map and LED wiring); `Firmware_Templates/src/board/disabled_wiring.rs` |
| Confidence | Probable. GPIO12 (MTDI) selects flash voltage at reset on classic ESP32 modules unless eFuses override it. The actual board, eFuse state and LED wiring were not inspected |
| Status | In Progress (Phase 0 partial; see phases/phase-0/RESULT.md §7) |
| Proposed phase | 2E (roadmap, pending approval) |

**What is wrong.** Wiring is authored inline in `main()` with no board description or conflict check. `led1` drives GPIO12, a strapping pin: if external circuitry holds it high at reset, the module selects 1.8 V flash and fails to boot on 3.3 V-flash modules. Commented module wiring reuses GPIO 12, 14 and 27, so uncommenting it collides with the LEDs. `Peripherals::take()` would make that a compile error in most cases, but not when the pins come from different code paths.

**Why this is a problem.** Hardware configuration is not reviewable in one place, and a wiring change requires editing control-flow code.

**How it could affect Pinora.** The board may fail to boot, depending on how LED1 is wired, and enabling modules produces confusing pin conflicts.

**Recommended solution.** Phase 0 adds a documented pin map, without changing any assignment. Later: a board/wiring module or table, and moving LED1 off GPIO12 (a behavior change, so explicit approval is needed).

**Risk of changing it.** Changing pins is a hardware change and requires rewiring.

**How we will verify the fix.** Review against the board schematic. Boot test with the LED wired as built.

---

## PIN-TPL-019 — Functional defects in inactive sensor modules (RFID, remote receiver, rangefinder)

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `Firmware_Templates/src/module/rfid.rs:226-262` vs `:264-279` (read collects up to 47 blocks; send requires exactly 16 bytes), `:381` (`WritePayload` appends without bound), `:82-85` (chip-version check disabled); `Firmware_Templates/src/module/remote_receiver.rs:127` (`.unwrap()` on GPIO read), `:121-140` (polling decode, see PIN-TPL-009); `Firmware_Templates/src/module/range_finder.rs:177-195` (Critical SysLog every failed tick, plus nested error logs) |
| Current location (after Phase 0) | `Firmware_Templates/src/modules/sensors/rfid.rs:222-275`, `:377`, `:78-81`; `Firmware_Templates/src/modules/sensors/remote_receiver.rs:127`; `Firmware_Templates/src/modules/sensors/rangefinder.rs:175-195` |
| Confidence | Confirmed (static); not executed |
| Status | Open |
| Proposed phase | 2C (roadmap, pending approval) |

**What is wrong.** RFID `read_card` concatenates every readable block, up to 752 bytes. `send_card_data` then insists on exactly 16 bytes, so any card with more than one readable block never produces a `GetCard` event. Repeated `WritePayload` commands grow `temp_write_data` without limit. The MFRC522 version check is commented out, so a missing reader appears initialized. The rangefinder emits two Critical SysLogs per failed tick at loop rate, which floods the egress queues (PIN-TPL-010).

**Why this is a problem.** These features cannot work as written once enabled.

**How it could affect Pinora.** RFID silently never reports cards, and memory can be exhausted by commands. A disconnected rangefinder floods every log.

**Recommended solution.** Define the RFID payload format, such as a UUID in block 1 or a length-prefixed record. Bound the write buffer. Restore the version check. Rate-limit sensor error reporting.

**Risk of changing it.** RFID card data format is a persisted-data format on physical cards, so it must be decided deliberately.

**How we will verify the fix.** Host tests with a mock MFRC522 interface (after Phase 1C). Hardware card read and write when authorized.

---

## PIN-TPL-020 — Frontend structure: a "god" store, very large module files and weakened typing

| Field | Value |
|---|---|
| Severity | Medium |
| Affected files | `UI/src/lib/Modulefront.TS` (385 lines: transport lifecycle, IPC listeners, registry, lookup map, logs, system info); `UI/src/lib/Modules/lidar.tsx` (948 lines), `stepper.tsx` (552), `rangefinder.tsx` (382), each combining Zod schema, Zustand store, IPC commands and React views; `UI/tsconfig.json:27` (`noImplicitAny: false`); `UI/src/lib/IncomingCommand.ts:4` (`command: any`); `UI/src/lib/Modules/Led.tsx:49` (`handleEvent: (event: any)`) |
| Current location (after Phase 0) | `UI/src/lib/Modulefront.ts` (381 lines); `UI/src/lib/Modules/lidar/view.tsx` (616), `stepper/view.tsx` (339); `UI/tsconfig.json:27`; `UI/src/lib/IncomingCommand.ts:7`; `UI/src/lib/Modules/Led/store.ts` (`handleEvent: (event: any)`) |
| Confidence | Confirmed |
| Status | In Progress (Phase 0 partial; see phases/phase-0/RESULT.md §7) |
| Proposed phase | 2D (roadmap, pending approval) |

**What is wrong.** One Zustand store owns unrelated concerns, and its actions call `invoke` directly. Module files mix protocol schemas, state, side effects and UI. `strict` is on, but `noImplicitAny` is off and outgoing commands are typed `any`, so the command schemas in `protocol/command.ts` are never enforced on send. Seven module UIs (servo, stepper, LiDAR, rangefinder, RFID, IMU, remote receiver) are compiled but never registered.

**Why this is a problem.** It is hard to test, hard to reason about, and easy to break when extending.

**How it could affect Pinora.** Every new module requires touching the god store and a large mixed file.

**Recommended solution.** Phase 0: split module files into `schema.ts` / `store.ts` / `View.tsx` per module without changing exports' behavior, and move the store's pure helpers (message classification, factory map) into separate modules. Later: type `IncomingCommand` with the inferred command type, enable `noImplicitAny`, and separate a transport service from the store.

**Risk of changing it.** Low for file splits if import paths are updated and behavior is unchanged. The store-shape redesign is behavioral and goes in a later phase.

**How we will verify the fix.** `tsc` and Vite build pass. Bundle contents compared before and after. A manual smoke test in `tauri dev`, when authorized.
