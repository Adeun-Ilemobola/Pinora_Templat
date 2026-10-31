# Remediation roadmap — Pinora Template

| Field | Value |
|---|---|
| Status | **Approved** 2026-10-31 (scope and order). Progress: 1A and 1B Completed — Verified; 1C next, awaiting authorization |
| Written | 2026-10-31, after the [post-Phase-0 reassessment](../reports/post-phase-0-reassessment.md) |
| Code basis | branch `phase-0/structural-reorganization` @ `a2204b33` |
| Findings covered | all 27 (0 Critical, 6 High, 13 Medium, 8 Low) |

Approving this roadmap approves the **scope and order** below. It does **not** start any phase. Each subphase gets its own `phases/phase-N/phase-Nx/PLAN.md` with exact file lists when it is authorized. The pattern for each one is: plan → approval → work → verification → `RESULT.md` → mirror sync → **stop**. Phase directories are created only when their phase is authorized.

## Order and the exceptions to severity order

| Order | Subphase | Severity addressed | Findings | Active image affected? |
|---|---|---|---|---|
| 1 | **1A** Firmware host-test harness and CI | Medium (enabler) | 008 | No (tests and CI only) |
| 2 | **1B** Firmware command safety | High | 001, 002, (025 Low) | **Yes** |
| 3 | **1C** Trust boundaries | High | 003 (pairing policy), 006 | **Yes** |
| 4 | **1D** Actuator and LiDAR safety | High | 004, 005 | No (latent modules) |
| 5 | **2A** Protocol contract v1 | Medium | 007, 012, 017, (021 Low) | Yes |
| 6 | **2B** Transport robustness and authentication | Medium | 010, 013, (016 Low), 003 (app-level auth) | Yes |
| 7 | **2C** Firmware runtime and sensor modules | Medium | 009, 019, (027 Low) | Partly |
| 8 | **2D** Desktop performance and structure | Medium | 011, 014, 020 | Yes |
| 9 | **2E** Build reproducibility and hardware configuration | Medium | 015, 018 | Build only / pins on approval |
| 10 | **3A** Low-severity cleanup and hardening | Low | 022, 023, 024, 026 | Yes (packaging) |

**Exceptions, explained:**

1. **1A (a Medium finding) comes before the High fixes.** Firmware logic cannot be tested on the host today. Without a harness, the 1B–1D fixes could only be checked by hardware testing, which this process restricts. 1A adds tests and CI only and changes no behavior.
2. **1B and 1C come before 1D**, although all three are High. 1B and 1C fix defects reachable in the shipped image (LEDs over Bluetooth). 1D fixes code that is not instantiated.
3. **Low findings ride along with the subphase that edits the same code:** 025 in 1B, 021 in 2A, 016 in 2B, 027 in 2C. Splitting them out would mean editing the same lines twice.
4. **PIN-TPL-003 is split.** The pairing policy in 1C needs no wire change. Application-level authentication needs a protocol handshake, so it waits for the versioned protocol in 2A and is built in 2B.
5. **2E has no technical dependency** on 2A–2D and could run earlier if you want fresh-clone builds sooner. It is listed last in Phase 2 only because 015 and 018 have the least runtime impact.

**Safety gate.** No servo, stepper or LiDAR module may be enabled in the default image until 1B, 1C and 1D are all **Completed — Verified**. This is the [Critical escalation watch-list](../findings/critical.md).

## Recurring rules (all subphases)

- Work on a new local branch per subphase, cut from the previous subphase's result, with one commit per step. Nothing is pushed. Merging to `main` is your decision.
- Golden wire-format tests must stay byte-identical unless the subphase is explicitly a versioned protocol change (2A, 2B). A changed golden file is then reviewed line by line.
- Only hardware-independent checks run automatically. Each subphase lists an optional hardware check that **you** run or separately authorize.
- `RESULT.md` separates pre-existing failures from new ones and states what is verified versus inferred.

---

## Phase 1A — Firmware host-test harness and CI

**Objective.** Make the pure firmware logic testable with `cargo test` on the host. Wire every test suite into one command and a CI workflow.

**Linked findings.** PIN-TPL-008, plus enabling verification for 1B–1D.

**Affected files and components.**
- New host-only test crate. The recommended location is a **new top-level directory** `tests/firmware-host/`. It needs your approval because contract C-10 freezes top-level layout; it sits outside `Firmware_Templates/` so that directory's `.cargo/config.toml` (Xtensa target, `build-std`) and `rust-toolchain.toml` do not apply.
- The crate includes hardware-free firmware files verbatim via `#[path]`: `utilities/math.rs`, `core/module.rs`, `core/transport/emitter.rs`, `core/transport/discovery_cache.rs`.
- `UI/package.json`: add a `"test": "bun test"` script. This is a manifest change; the lockfile is unaffected.
- `justfile`: add a `test` recipe. This is additive; existing recipe names are unchanged.
- New `.github/workflows/ci.yml`.

**Implementation strategy.**
1. Create the host crate with a minimal `crate::core::…` module shim so the included files compile unchanged.
2. Write characterization tests for current behavior: the `range_*` overflow behavior (as `#[should_panic]` in debug), `TransportEmitter` reliable vs lossy semantics (full queue, disconnected receiver), and `discovery_cache::remember` (replace-in-place, ordering).
3. Add the `test` script and recipe.
4. Add the CI jobs: `cargo fmt --check` (3 packages); protocol `test` and `clippy`; host tests; Tauri `clippy` on `windows-latest`, which needs `wifi_config.rs` copied from the example inside CI; `tsc --noEmit`; `bun test`; Vite build. The firmware build is excluded (toolchain size) and noted as a later optional job.

**Behavioral impact.** None. CI only takes effect if you push the branch.

**Risk assessment.** `#[path]` includes tie the test crate to file locations, and a move breaks the build loudly, which is acceptable. CI may need iteration on runner setup. Adding a top-level directory could matter to external tooling (C-10, unknown consumers).

**Verification strategy.** All suites pass locally. Deliberately altering the golden file, or one included source line, makes the relevant test fail. Firmware debug and release builds are unchanged in size. CI YAML is validated by running the same commands locally.

**Completion criteria.** One command (`just test`) runs protocol, host-firmware and frontend tests green. The CI workflow is committed. The register marks 008 as In Progress (harness done) and records what remains for 1D/2C.

**Recovery plan.** All changes are additive. Revert the subphase commits or delete the branch.

**Dependencies.** Phase 0 (complete).

---

## Phase 1B — Firmware command safety

**Objective.** A bad or failing command must never stop the firmware or silently mis-drive an output. Events must report the values actually applied.

**Linked findings.** PIN-TPL-001, PIN-TPL-002, PIN-TPL-025.

**Affected files and components.**
- `Firmware_Templates/src/runtime/mod.rs`, plus a new `runtime/dispatch.rs` (pure and host-testable).
- `core/module.rs`, where `ModuleError` may become typed.
- `modules/actuators/led.rs`, `utilities/math.rs`.
- `modules/actuators/servo.rs` and `stepper.rs`, for the event values only.

**Implementation strategy.**
1. Extract command dispatch into a pure function. A handler `Err` is logged and reported as a `SysLog` (an existing wire shape, from the module's id), and the loop continues.
2. Report tick errors as rate-limited `SysLog` messages per module instead of discarding them.
3. LED: clamp `state` to 0–100. **Decision for you:** clamp and warn (recommended) or reject. Then emit the applied level.
4. Make the `range_*` helpers overflow-safe, by widening intermediates and saturating.
5. Servo and stepper limit events report the stored values.
6. Replace the remaining `.unwrap()` calls in active paths with error propagation into the new dispatch path.

**Behavioral impact.** Yes, intended:
- Failed commands no longer stop the firmware.
- Out-of-range LED values saturate.
- New `SysLog` messages appear.
- Limit events report applied values.

No new message types are added, so the golden file is unchanged.

**Risk assessment.** More `SysLog` traffic, which is mitigated by rate limiting. Anyone relying on "error stops the device" loses that crude stop; the safe-state concept arrives in 1D.

**Verification strategy.**
- Host tests: dispatch continues after an `Err` and emits exactly one `SysLog`; the LED mapping is right at 0, 100, 101, 524 353 and `u32::MAX`; math boundary tests.
- The golden tests are unchanged.
- Firmware debug and release builds pass.
- Optional hardware check: send `SetState{state: 600000}`. The LED should be at full brightness, report 100, and not reboot. Then force a handler error and confirm later commands still work.

**Completion criteria.** The tests above pass and builds are green. 001, 002 and 025 are **Resolved**, and become **Verified** after the hardware check.

**Recovery plan.** Revert the subphase commits. The pre-1B behavior is restored exactly; the golden file is unchanged.

**Dependencies.** 1A.

---

## Phase 1C — Trust boundaries

**Objective.** Stop arbitrary nearby devices from pairing, and stop malformed input from crashing the desktop.

**Linked findings.** PIN-TPL-003 (pairing-policy part), PIN-TPL-006.

**Affected files and components.**
- `Firmware_Templates/src/core/transport/bluetooth.rs`; `Firmware_Templates/BLUETOOTH_TRANSPORT.md`.
- `UI/src-tauri/src/serial_transport.rs`, plus a new line-classifier module.
- `UI/src/lib/Modulefront.ts`, `UI/src/lib/logs.ts`, `UI/src/App.tsx` (route `errorElement`).
- `UI/tests/logs.test.ts`: the PIN-TPL-006 expectations flip deliberately.

**Implementation strategy.**
1. **Bluetooth.** Discoverable and accepting SSP only during a window after boot (**decision for you:** length; 120 s is suggested). After the window: connectable, not discoverable, and accepting bonded peers only. Document how to re-open the window (power-cycle). There is no wire change.
2. **Desktop receive path.** Tauri validates every COM line as a Rust `ProtocolMessage`, as the Wi-Fi path already does. Valid lines go out as `espState`. Other lines go on a **new additive event** `espConsole` (**decision for you:** add `espConsole`, recommended, or drop the lines).
3. Make `describeLog` and `matchesLog` total, with an "Unrecognized message" fallback.
4. Add an error boundary, so a render fault is contained to one route.

**Behavioral impact.** Yes:
- New devices can pair only during the window.
- Firmware console lines stop arriving on `espState`.
- Malformed messages show as unrecognized instead of crashing the view.

**Risk assessment.** A missed pairing window locks out new devices until reboot (documented). An existing Windows pairing may need re-pairing if the bond store changes. Validation drops unknown variants unless they parse; the Rust types are authoritative.

**Verification strategy.**
- Tauri unit tests for the line classifier (valid, unknown variant, console text, oversized line).
- `bun test`, with the updated expectations.
- A component test of the error boundary.
- Optional hardware check: pair inside the window and connect, then try a second device after the window (it must fail), then reconnect the bonded desktop.

**Completion criteria.** Tests and builds are green. 006 is **Resolved**. 003 is **In Progress**, with the pairing policy done and app-level auth pending in 2B.

**Recovery plan.** Revert the commits. Bonds made during testing can be cleared from Windows.

**Dependencies.** 1A. Independent of 1B, so the two can be reviewed in either order.

---

## Phase 1D — Actuator and LiDAR safety (latent modules)

**Objective.** Make servo, stepper and LiDAR safe to enable: no uncommanded motion, bounded targets, terminating scans and bounded memory.

**Linked findings.** PIN-TPL-004, PIN-TPL-005.

**Affected files and components.**
- `Firmware_Templates/src/modules/actuators/{servo,stepper}.rs`, `modules/composite/lidar.rs`.
- New small output traits so the state machines can run on the host, for example `modules/actuators/outputs.rs`.
- `board/disabled_wiring.rs` (documentation), host tests in `tests/firmware-host/`.

**Implementation strategy.**
1. Constructors leave hardware stationary. The servo self-test becomes an explicit board-level function that is off by default; adding a self-test *command* is a wire change and is deferred to 2A.
2. The stepper starts `Idle`, not `Homing`.
3. Validate every target: finite, and within configured mechanical limits.
4. De-energize stepper coils when idle (**decision for you:** default on, or a per-instance holding option).
5. LiDAR: terminate when the limit is reached *or passed*. Require `step ≥ 1`. Validate the ROI against the servo limits. Flush points by chunk size, cap `point_map`, and use checked arithmetic.
6. Replace `.unwrap()` calls with errors that go into the 1B dispatch path.
7. Define a documented "safe state" for each actuator.

**Behavioral impact.** Yes, in latent modules only. The active LED image is unaffected.

**Risk assessment.** Scan chunk numbering changes, and the desktop LiDAR view consumes it (the view is unrouted). Introducing the traits touches every actuator file.

**Verification strategy.**
- Host tests of the motion and scan state machines with fake outputs: no output writes during construction; out-of-range and non-finite targets rejected; scans terminate for ROI/step combinations, including the one in the finding; buffer size bounded.
- Firmware builds pass.
- Optional hardware check, **only with explicit authorization and supervision:** enable one servo on a bench and confirm no motion at boot.

**Completion criteria.** Tests and builds are green, and 004 and 005 are **Resolved**. The safety gate opens only after 1B, 1C and 1D are all Verified.

**Recovery plan.** Revert the commits. The latent modules return to their previous (unsafe) state, still not instantiated.

**Dependencies.** 1A (harness), 1B (error propagation path).

---

## Phase 2A — Protocol contract v1

**Objective.** One enforced, versioned contract between firmware and desktop, with rediscovery and current-state replay.

**Linked findings.** PIN-TPL-007, PIN-TPL-012, PIN-TPL-017, PIN-TPL-021.

**Affected files and components.**
- `protocol/src/**`, plus the golden file, which changes deliberately and is versioned.
- `UI/src/lib/protocol/*`, `UI/src/lib/Modules/*/schema.ts`.
- Firmware `runtime/`, `core/module.rs`, `modules/*` (`first_emit` state).
- Tauri `serial_transport.rs` (USB disconnect event).
- `UI/src/lib/Modulefront.ts`.

**Implementation strategy.**
1. Shared fixtures first. The Rust golden test exports fixtures that the TypeScript schemas must parse in `bun test`. This needs no new dependency.
2. Then evaluate code generation (`ts-rs`/`specta`), which needs approval for new dependencies.
3. Add `protocol_version` to the `System` snapshot, using `serde(default)` so old firmware still parses.
4. Add a proper system command for rediscovery to replace the `"SSI"` stand-in. Every module reports current state in `first_emit`.
5. Registry: route commands to children, enforce unique lookup names, and document reserved IDs.
6. Desktop: a USB disconnect event, clearing state *before* connecting, and an "unknown until reported" module state.
7. Correct misspelled wire names with `serde(alias)` on input, and switch output names at the new version (**decision for you:** fix now, or keep for compatibility).

**Behavioral impact.** Yes. This is a versioned wire change. Old desktops paired with new firmware, and the reverse, must be tested.

**Risk assessment.** It breaks any external consumer of the wire format (C-01; `pinora_cli`/`studio` are unknown). Mitigations are aliases and the version field.

**Verification strategy.**
- The golden diff is reviewed and approved.
- Cross-version tests: the old golden parses with the new code where compatibility is promised.
- TypeScript fixture tests.
- Optional hardware check: reconnect and see current LED state; unplug USB and see the UI's disconnected state.

**Completion criteria.** Tests are green and the contract is documented as v1. 007, 012, 017 and 021 are Resolved.

**Recovery plan.** Revert. The protocol returns to its unversioned v0 format, and the golden file is restored.

**Dependencies.** 1C (validated receive path).

---

## Phase 2B — Transport robustness and authentication

**Objective.** Predictable delivery under load, robust Wi-Fi start-up and listener behavior, one framing implementation, and real application-level authentication.

**Linked findings.** PIN-TPL-010, PIN-TPL-013, PIN-TPL-016, PIN-TPL-003 (remaining part).

**Affected files and components.**
- Firmware: `core/transport/{emitter,manager,bluetooth,wifi,serial}.rs`, a new framing module.
- Tauri: `serial_transport.rs`, `wifi_transport.rs`.
- `protocol/` (handshake messages).

**Implementation strategy.**
1. One framing helper per language, with a single limit table.
2. Emission priorities: rate-limit and coalesce per module, and report drop counters periodically.
3. Wi-Fi: separate reader and writer, and bring Wi-Fi up in the background with retry instead of failing boot.
4. Listener: accept on one thread and authenticate on a worker with an overall deadline.
5. A challenge-response handshake (HMAC with a per-device key provisioned into NVS) for Wi-Fi and Bluetooth. This needs a crypto dependency, so **decision and approval** are required.

**Behavioral impact.** Yes: timing and order of telemetry, Wi-Fi start-up, the authentication workflow and provisioning.

**Risk assessment.** Provisioning UX. Key storage. Cross-platform crypto dependencies in firmware.

**Verification strategy.**
- Host tests of the framing and policy logic.
- Tauri tests with slow and malicious peers.
- A synthetic flood test (emitted vs delivered counts).
- Optional hardware check: Wi-Fi with no AP at boot; LiDAR-rate telemetry.

**Completion criteria.** Tests are green. 010, 013 and 016 are Resolved, and 003 is Resolved.

**Recovery plan.** Revert. Devices keep working with the v1 protocol without the handshake if the handshake is behind a version gate.

**Dependencies.** 2A.

---

## Phase 2C — Firmware runtime and sensor modules

**Objective.** A defined scheduling model, deterministic module order, and working sensor modules.

**Linked findings.** PIN-TPL-009, PIN-TPL-019, PIN-TPL-027.

**Affected files and components.**
- `Firmware_Templates/src/runtime/`, `core/module.rs` (ordered registry), `core/hardware/timing.rs`.
- `modules/sensors/{rfid,remote_receiver,rangefinder}.rs`.

**Implementation strategy.**
1. An ordered registry: a `Vec` in construction order plus an index for lookup.
2. A scheduling model (**decision for you:** fixed-rate tick with `vTaskDelayUntil`, or block on the command channel until the next deadline).
3. The IR receiver uses the RMT peripheral instead of polling.
4. RFID: define the card payload format (**decision for you**; it is a persisted format), bound the write buffer, and restore the chip check.
5. Rate-limit rangefinder errors using the 2B policy.

**Behavioral impact.** Yes: timing, ordering, CPU use, and sensor behavior.

**Risk assessment.** Timing changes affect every module. The RFID format is irreversible on cards already written.

**Verification strategy.**
- Host tests for order and the RFID buffer logic.
- On-device instrumentation of loop period and jitter (a histogram over `log`).
- Optional hardware check for the IR and RFID modules.

**Completion criteria.** Measured jitter is within the target set in the 2C plan, and 009, 019 and 027 are Resolved.

**Recovery plan.** Revert the commits.

**Dependencies.** 1A, 1B; 2B (rate-limit policy).

---

## Phase 2D — Desktop performance and structure

**Objective.** The desktop stays responsive in long, high-rate sessions, with typed, testable state management.

**Linked findings.** PIN-TPL-011, PIN-TPL-014, PIN-TPL-020.

**Affected files and components.**
- `UI/src/lib/Modulefront.ts` (split into a transport service and store slices), `UI/src/components/LogViewer.tsx`, `UI/src/lib/logs.ts`, `UI/src/main.tsx`, `UI/src/lib/IncomingCommand.ts`, `UI/tsconfig.json`.
- `UI/src-tauri/src/commands.rs`, `bluetooth_ports.rs`.

**Implementation strategy.**
1. Logs: a bounded ring buffer with export, batched state updates, and precomputed search text. Remove per-message `console.info` forwarding.
2. Tauri commands: make them `async` or use `spawn_blocking`, keeping command names. Cache Bluetooth port metadata.
3. Separate the transport service from the Zustand store. Type `IncomingCommand` with the inferred command types. Enable `noImplicitAny` incrementally.

**Behavioral impact.** Yes: old log entries beyond the bound are dropped (export is available) and UI responsiveness changes. IPC names are unchanged.

**Risk assessment.** Store refactors can break subscriptions. The characterization tests and component tests mitigate this.

**Verification strategy.**
- A synthetic 100 000-message test: memory stays bounded and append cost is flat.
- React profiler timings.
- The main-thread stall during port refresh is measured before and after.

**Completion criteria.** 011, 014 and 020 are Resolved.

**Recovery plan.** Revert the commits.

**Dependencies.** 2A for typing (020). 011 and 014 have no dependency and may be pulled earlier on request.

---

## Phase 2E — Build reproducibility and hardware configuration

**Objective.** A fresh clone builds on any supported machine, and the board configuration is explicit.

**Linked findings.** PIN-TPL-015, PIN-TPL-018.

**Affected files and components.** `protocol/src/lib.rs` + `protocol/build.rs` (`wifi_config` fallback), `Firmware_Templates/Cargo.toml`, `Firmware_Templates/.cargo/config.toml`, `Firmware_Templates/Cargo.lock` (dependency removal only), `.gitignore`, `Firmware_Templates/src/board/`.

**Implementation strategy.**
1. Generate a placeholder `wifi_config` when the local file is absent, with a build warning.
2. Pin `rev` on the ESP Git patches to the commits already locked.
3. Use a relative `target-dir` with an environment override, so `C:/t` keeps working.
4. Remove the unused `rand` and `embassy-time` (D4, revisited).
5. A board configuration table.
6. **Decision for you:** move LED1 off strapping pin GPIO12, which requires rewiring.

**Behavioral impact.** Build only, unless the pin change is approved.

**Risk assessment.** The lockfile changes for the dependency removal (contract C-19, so approval is needed). Changing the cache path invalidates incremental builds.

**Verification strategy.** A clean-clone build in a temporary directory following only the documented steps. Firmware ELF code sizes compared, which are expected unchanged apart from the removed dependencies.

**Completion criteria.** 015 and 018 are Resolved (018 Verified after you rewire, if you choose to).

**Recovery plan.** Revert the commits. The lockfile is restored from Git.

**Dependencies.** None technical. It can be scheduled earlier on request.

---

## Phase 3A — Low-severity cleanup and hardening

**Objective.** Zero-warning builds enforced in CI, a hardened packaged app, accurate documentation, and enforced peripheral ownership.

**Linked findings.** PIN-TPL-022, PIN-TPL-023, PIN-TPL-024, PIN-TPL-026.

**Affected files and components.**
- All three Rust packages (warnings), CI workflow, TypeScript lint config (new dev dependencies, so a lockfile change needing approval).
- `UI/src-tauri/tauri.conf.json`, `Cargo.toml` metadata, `commands.rs`.
- Documentation: `README.md`, `.espConfig`, `pre-script.rhai` (only after the sibling-project audits clear C-10).
- `Firmware_Templates/src/core/hardware/console.rs` + `main.rs`.

**Implementation strategy.**
1. Fix the warnings and enable `-D warnings` in CI.
2. Add a CSP.
3. Product name and identifier (**decision for you:** the identifier change moves the app-data directory).
4. Remove the unused `greet`, an IPC change the frontend does not use.
5. `start_port` returns error detail, an IPC return-shape change with a matching frontend update.
6. Fix the documentation drift.
7. Consume `p.uart0` explicitly.

**Behavioral impact.** Packaging and IPC error detail change. There is no runtime change to the firmware.

**Risk assessment.** CSP can break Vite dev mode if it is too strict. The identifier change relocates saved settings.

**Verification strategy.** Builds have zero warnings. Packaged-app inspection. A CSP violation check in devtools. A Serial transport smoke test after the UART change (optional, on hardware).

**Completion criteria.** 022, 023, 024 and 026 are Resolved.

**Recovery plan.** Revert the commits.

**Dependencies.** 1A (CI). Documentation for C-10 items waits for the sibling-project audits.

---

## Approval requested

Approve this roadmap (scope and order), and authorize **Phase 1A — Firmware host-test harness and CI**. That includes the decision to add the top-level `tests/firmware-host/` directory and a CI workflow file.
