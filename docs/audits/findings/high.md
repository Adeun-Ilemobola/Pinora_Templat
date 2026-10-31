# High-severity findings

Part of the [issue register](README.md). IDs are stable; records are kept after resolution.
Source paths are repository-relative to the `Pinora_Templat` root, with line numbers taken at commit `5b5c8ed4`.

There are no Critical findings (see [critical.md](critical.md) for why). Two of the findings below (PIN-TPL-004, PIN-TPL-005) are in code that the current firmware image does not instantiate. They are rated High, not Critical, for that reason only. **They become Critical as soon as that code is wired into `main.rs`.**

---

## PIN-TPL-001 — A single failed command stops the firmware main loop

| Field | Value |
|---|---|
| Severity | High |
| Affected files | `Firmware_Templates/src/main.rs:272-295` (main loop), `:279` (`?` on handler), `:274` (tick errors discarded); `Firmware_Templates/src/core/modulecore.rs:92` (`handle_command` returns `anyhow::Result`) |
| Current location (after Phase 0) | `Firmware_Templates/src/runtime/mod.rs:38-67` (tick `:45`, handler `?` `:50`); `Firmware_Templates/src/core/module.rs` (`handle_command` contract) |
| Confidence | Confirmed (static: the loop exits). Probable: the device state after exit, which was not observed on hardware |
| Status | **Resolved** in Phase 1B (2026-10-31); Verified after the hardware check in phases/phase-1/phase-1b/RESULT.md (B-V6) |
| Proposed phase | 1B (roadmap, pending approval) |

**What is wrong.** The main loop calls `module.handle_command(&command.command)?`. When any module's handler returns `Err`, the `?` returns from `main()`. Errors from `module.tick()` go the opposite way: `let _ =` discards them with no log or event.

**Why this is a problem.** One recoverable fault, such as an I²C NACK in a servo command or a failed sensor reconfiguration, permanently ends command processing. Nothing restarts the loop and the desktop is not told. On ESP-IDF, returning from Rust `main` returns from `app_main`, and the main task is deleted. It does not reboot. Locals are dropped as `main` exits: `LedcDriver::drop` calls `ledc_stop`, so the LEDs go to their idle level, and the module registry is destroyed. Transport threads may survive with no consumer for their command queue. The Bluetooth worker returns once `incoming.send` fails.

**How it could affect Pinora.** The device can stay paired or connected and look healthy while it ignores every command. Any actuator driven by an external controller, such as a PCA9685 servo, keeps its last commanded output. Tick errors are discarded, so a sensor that fails every tick is invisible unless the module also emits its own SysLog.

**Evidence.**

```rust
// Firmware_Templates/src/main.rs:273-279
for module in modules.values_mut() {
    let _ = module.tick();                         // errors discarded
}
if let Ok(command) = command_receiver.try_recv() {
    if let Some(module) = modules.get_mut(&command.id) {
        module.handle_command(&command.command)?;  // any Err exits main()
```

`esp-idf-sys` `app_main` simply calls `main(…)` and returns (`esp-idf-sys/src/start.rs`, locked revision `33045844`). `esp-idf-hal` `LedcDriver::drop` calls `ledc_stop` (`esp-idf-hal/src/ledc.rs`, locked revision `846cff17`).

**Plain-English explanation.** If one command fails, the board stops listening to every later command and gives no sign that it has done so.

**Recommended solution.** Treat handler errors as per-command failures. Log them, emit a `SysLog` (an existing event shape) that names the module, and keep looping. Decide on a policy for repeated tick failures, such as rate-limited reporting, and for truly unrecoverable states. A deliberate `esp_restart()` after putting actuators in a safe state is better than a silent half-alive device. Consider a typed `ModuleError` in place of `anyhow` + `From<()>` (`modulecore.rs:40`).

**Risk of changing it.** This changes error semantics, which Phase 0 forbids, so it belongs in Phase 1. Some callers may rely on "fail stops everything" as a crude safety stop. That reliance must be replaced by an explicit safe-state routine, not just removed.

**How we will verify the fix.** Host-side unit test of the dispatch function with a module stub whose handler returns `Err`: the loop continues and a SysLog is produced. On hardware, when authorized, force an I²C failure and confirm that later LED commands still work.

---

## PIN-TPL-002 — LED brightness command is not range-checked; arithmetic overflow corrupts output or reboots the board

| Field | Value |
|---|---|
| Severity | High |
| Affected files | `Firmware_Templates/src/module/ledmodule.rs:42-46`; `Firmware_Templates/src/utilities/math.rs:1-13` (`range_u32`); `protocol/src/module/ledmodule.rs` (`SetState { state: u32 }`) |
| Current location (after Phase 0) | `Firmware_Templates/src/modules/actuators/led.rs:46-50`; `Firmware_Templates/src/utilities/math.rs:8-20` |
| Confidence | Confirmed (probe executed against the real `math.rs` and protocol crate; see [validation baseline](../evidence/validation-baseline.md#dynamic-probes)) |
| Status | **Resolved** in Phase 1B (2026-10-31); Verified after the hardware check in phases/phase-1/phase-1b/RESULT.md (B-V6) |
| Proposed phase | 1B (roadmap, pending approval) |

**What is wrong.** `SetState { state }` accepts any `u32`. `set_state` computes `range_u32(state, 0, 100, 0, max_duty)` with no clamp, which is `(state - 0) * 8191 / 100`. It then reports `Brightness { level: state }`, echoing the unvalidated value.

**Why this is a problem.**
- For `state > 524 352`, the multiplication overflows `u32`. In a **debug** firmware build (`cargo build`, where overflow checks are on by default), that is a panic, which with `panic_abort` resets the board. Any connected peer can therefore reboot the device remotely.
- In a **release** build the value wraps silently. `state = 524 353` gives duty `81` (≈1 %), while the firmware reports brightness `524353`.
- For `101 ≤ state ≤ 524 352`, `esp-idf-hal` clamps the duty to maximum, but the reported level is still the out-of-range input.

**How it could affect Pinora.** The desktop and firmware disagree about the physical output. A malformed command reboots debug builds. Combined with PIN-TPL-003, any nearby paired device can trigger the reboot. The same unclamped mapping pattern is copied for other actuators (see PIN-TPL-004).

**Evidence (probe output).**

```
LED state=4e9 deserializes: true
debug:   range_u32(524353) PANICS (overflow)
release: range_u32(524353) = 81        range_u32(4000000000) = 19894661
```

**Plain-English explanation.** Sending "brightness 600000" instead of 0–100 either reboots the board or sets a random brightness, and the board then claims it set 600000.

**Recommended solution.** Clamp or reject `state` outside `0..=100` in firmware, which is the trust boundary. Emit the *applied* level. Make `range_*` helpers overflow-safe, using widened intermediates or `checked_`/`saturating_` arithmetic, and add unit tests. Optionally mirror the bound in the TypeScript schema (`z.number().int().min(0).max(100)`).

**Risk of changing it.** This changes behavior: out-of-range commands that used to produce wrapped duty will now saturate or be rejected. No known client sends values over 100; the UI slider is bounded 0–100.

**How we will verify the fix.** Unit tests for `range_u32`/`range_i32`/`range_i16` at the boundaries and at overflow values. A firmware dispatch test that `SetState{state: u32::MAX}` yields level 100, or a rejection SysLog, without panicking.

---

## PIN-TPL-003 — Bluetooth SPP accepts any nearby device: permanent discoverability, auto-confirmed pairing, no application-level authorization

| Field | Value |
|---|---|
| Severity | High |
| Affected files | `Firmware_Templates/src/core/transport/bluetooth.rs:85-92` (auto-confirm SSP), `:128-136` (IO capability `NONE`), `:138` (server start), `:208` (discoverable, never reverted) |
| Current location (after Phase 0) | `Firmware_Templates/src/core/transport/bluetooth.rs:104-110` (auto-confirm), `:162-171` (IO capability), `:173` (server), `:243` (discoverable) |
| Confidence | Confirmed (static). Exploitation not attempted; no radio testing was performed |
| Status | Open |
| Proposed phase | 1C (+2B) (roadmap, pending approval) |

**What is wrong.** The firmware advertises itself as discoverable for its whole uptime and declares no input/output capability, so Secure Simple Pairing uses Just Works. It then answers every `PairingUserConfirmationRequest` with `true`. The SPP link is encrypted, but encryption comes from that unauthenticated pairing. After connection, any well-formed `IncomingCommand` is executed: there is no token, allow-list or bonding check. Wi-Fi by contrast requires a shared token, though a weak one (PIN-TPL-013).

**Why this is a problem.** Any Bluetooth Classic device in radio range can pair, open SPP while the desktop is not connected, and drive every registered module. The encryption gives no protection against an active man-in-the-middle during pairing, and the README already acknowledges this.

**How it could affect Pinora.** Today the image drives three LEDs, so the impact is modest. Through PIN-TPL-002 it is also a remote reboot of debug builds. Pinora is a template for servo, stepper and LiDAR control, though. Once actuators are registered, this becomes **unauthenticated remote control of moving hardware**, and the rating should rise to Critical.

**Evidence.**

```rust
// bluetooth.rs:85-92
GapEvent::PairingUserConfirmationRequest { bd_addr, .. } => {
    // The development board has no input or display: SSP Just Works.
    esp!(unsafe { esp_bt_gap_ssp_confirm_reply(&bd_addr as *const _ as *mut _, true) })
// bluetooth.rs:208
gap.set_scan_mode(true, DiscoveryMode::Discoverable)
```

**Plain-English explanation.** Anyone nearby with a laptop or phone can pair with the board and send it commands. Nothing asks whether they are allowed to.

**Recommended solution (to be designed in Phase 1B; options, not decided).**
1. Gate discoverability, for example only for N seconds after boot or after a physical button press, and switch to connectable-only once bonded.
2. Accept only previously bonded addresses after first pairing. ESP-IDF stores bonds in NVS.
3. Add an application-level handshake equivalent to the Wi-Fi token, ideally a per-device secret provisioned out of band.
4. Long-term, numeric-comparison pairing if the board gains a display or button.

**Risk of changing it.** Pairing workflow, the Windows setup guide and the desktop connection flow all change. The protocol contract (PIN-TPL-007) changes if a handshake is added. Existing bonds may need re-pairing.

**How we will verify the fix.** Hardware test, when authorized: an un-bonded second device cannot pair or connect after the provisioning window, and the bonded desktop still connects. Host-side test for any handshake parser.

---

## PIN-TPL-004 — Actuator modules move without being commanded and accept unbounded targets *(inactive in current image)*

| Field | Value |
|---|---|
| Severity | High (rises to Critical when servo or stepper modules are instantiated) |
| Affected files | `Firmware_Templates/src/module/servomodule.rs:51-58` (test sweep in constructor), `:74` (`.unwrap()` on I²C), `:157-169` (pivot setters); `Firmware_Templates/src/module/stepper.rs:111` (starts in `Homing`), `:142-144`, `:304-310` (`MoveToAngle` unbounded), `:311-322` / `:372-377` (pivot limits unvalidated), `:148` (coils stay energized when idle), `:255-267` (`.unwrap()` on GPIO); `Firmware_Templates/src/module/lidar.rs:166-179` (`initialize` sweeps both servos in `first_emit`) |
| Current location (after Phase 0) | `Firmware_Templates/src/modules/actuators/servo.rs:53-60` (test sweep), `:76` (`.unwrap()`), `:158-166`; `Firmware_Templates/src/modules/actuators/stepper.rs:118` (starts Homing), `:155`, `:262` (`.unwrap()`), `:310-328`, `:378`; `Firmware_Templates/src/modules/composite/lidar.rs:176` (`initialize`) |
| Confidence | Confirmed (static). No hardware was moved during this audit |
| Status | Open |
| Proposed phase | 1D (roadmap, pending approval) |

**What is wrong.**
- `ServoModule::new` immediately drives the servo to the offset, then to 35°, 10°, −10°, −35° and back to 0°, with no delay and no command. Every servo constructed, including the two inside LiDAR, jerks at boot. LiDAR's `first_emit` then sweeps both servos to the ROI corners.
- `StepperMotor::new` starts in `StepperState::Homing { cycle: 1 }`, so the motor starts sweeping ±90° three times as soon as ticking begins. There is no homing sensor, so "homing" here is open-loop motion.
- `MoveToAngle { angle: f32 }`, `SetPivotMax`/`SetPivotMin` accept any finite `f32` with no travel limit. A large angle produces effectively endless rotation. Above 2²⁴ steps, `step + 1.0 == step` in `f32`, so the target can never be reached and the coils stay energized.
- The stepper never de-energizes its coils when idle, which for ULN2003/28BYJ-48 style drivers means continuous holding current and heat. Probable; this depends on the driver.
- `.unwrap()` on I²C and GPIO results inside actuator paths turns a bus glitch into a panic, then a reboot, which replays the boot-time motion.

**Why this is a problem.** Motion nobody asked for at power-up, after every reboot and after every panic-reset is the classic way embedded rigs injure fingers or break mechanisms. The template is meant to be copied, and these behaviors would be copied with it.

**How it could affect Pinora.** Uncommenting the servo, stepper or LiDAR wiring in `main.rs` produces immediate motion with no operator action. A malformed or malicious command can request unbounded travel; see PIN-TPL-003 for who can send commands.

**Evidence.** `servomodule.rs:51-58`:
```rust
s.set_offset(s.offset)?;
s.set_angle(0)?;
let testrang: [i32; 4] = [35, 10, -10, -35];
for f in testrang { s.set_angle(f)?; }
s.set_angle(0)?;
```
`stepper.rs:111`: `mode: StepperState::Homing { cycle: 1 },`

**Plain-English explanation.** If someone turns the servo or stepper code back on, motors will start moving on their own at power-up. They will also go as far as any command tells them, with no limit.

**Recommended solution.** Construct actuators in a safe, stationary state. Make any self-test an explicit, separately authorized command. Validate every motion target against configured mechanical limits, and reject non-finite or out-of-range values. Add a de-energize-on-idle policy for steppers. Replace `.unwrap()` with error propagation to the (fixed) dispatch path of PIN-TPL-001. Document a "safe state" contract for every actuator.

**Risk of changing it.** This changes behavior. Anyone relying on the boot sweep as a wiring check loses it, so offer it as an explicit command.

**How we will verify the fix.** Host-side unit tests on the motion state machines, which requires the hardware-abstraction seam from Phase 1C. Then a supervised hardware test, only with explicit authorization: no motion at boot, and out-of-range commands rejected.

---

## PIN-TPL-005 — LiDAR scan can never finish and grows memory without bound *(inactive in current image)*

| Field | Value |
|---|---|
| Severity | High |
| Affected files | `Firmware_Templates/src/module/lidar.rs:244-251` (exact-equality termination), `:279-281` (flush only at row end), `:339-341` (ROI arithmetic), `:348-350` (`SetStep` accepts 0) |
| Current location (after Phase 0) | `Firmware_Templates/src/modules/composite/lidar.rs:254-261`, `:289`, `:349`, `:358` |
| Confidence | Confirmed by code reasoning. Not executed: the module is not instantiated and has no test harness |
| Status | Open |
| Proposed phase | 1D (roadmap, pending approval) |

**What is wrong.** Rows and scans end only when `curr_point_bottom.x == limit_point.x` (and the same for `y`). The position moves by `step` per tick and is clamped by the servos. If `step` does not divide the ROI span exactly, the position passes the limit, gets clamped at the servo bound instead, and never equals the limit. `SetStep { step: 0 }` means it never moves at all. Every tick in `Scanning` pushes a point into `point_map`, which is flushed only at the end of a row. `Roi` computes `max.x.abs() as u32 * 2 * …`, where `i32::MIN.abs()` panics in debug, and it does not validate `min < max`.

**Why this is a problem.** A scan with a typical ROI and step can sweep forever while the point buffer grows until the heap is exhausted. On ESP-IDF that means an allocation failure, then an abort and reboot, which then replays the boot-time servo motion of PIN-TPL-004.

**How it could affect Pinora.** The experimental LiDAR feature will appear to hang and then reboot the board when it is re-enabled.

**Evidence.**
```rust
// lidar.rs:250-251
let row_finished = self.curr_point_bottom.x == self.limit_point.x;
let scan_finished = row_finished && self.curr_point_bottom.y == self.limit_point.y;
```
Worked example: ROI x from 90 to −50 with step 3. That is 140/3 steps, which is not an integer, so x goes 90, 87, …, −48, −51, and keeps going to the −90 clamp. It never equals −50.

**Plain-English explanation.** For many settings the scanner never realizes it has reached the end. It keeps scanning and filling memory until the board crashes.

**Recommended solution.** Terminate on "reached or passed the limit" (`>=`/`<=` by direction). Validate `step ≥ 1`, and validate the ROI bounds against the servo limits. Flush by chunk size regardless of row boundaries and cap `point_map`. Compute totals with checked arithmetic.

**Risk of changing it.** This changes scan geometry and chunk numbering, and the desktop LiDAR view consumes chunk indices.

**How we will verify the fix.** Host-side unit tests of the scan state machine across ROI and step combinations (requires PIN-TPL-008 infrastructure): the scan terminates within the expected number of ticks and buffer size stays bounded.

---

## PIN-TPL-006 — Malformed device input crashes the desktop UI's log view

| Field | Value |
|---|---|
| Severity | High |
| Affected files | `UI/src/lib/Modulefront.TS:256-264` (`JSON.parse` then unvalidated `HandleEvent`), `:294` (appends any object to `logs`); `UI/src/lib/logs.ts:52`, `:65` (assumes shape); `UI/src/components/LogViewer.tsx:46`, `:117` (calls `describeLog`/`matchesLog` during render); `UI/src-tauri/src/lib.rs:156-164` (COM reader forwards raw lines) |
| Current location (after Phase 0) | `UI/src/lib/Modulefront.ts:234-264` (listener, `JSON.parse` `:252`), `:290`; `UI/src/lib/logs.ts:52`, `:65`; `UI/src/components/LogViewer.tsx:46`, `:117`; `UI/src-tauri/src/serial_transport.rs:170-178` |
| Confidence | Confirmed: probe executed (`describeLog` and `matchesLog` throw `TypeError`). The end-to-end UI effect is Probable: no error boundary or `errorElement` exists, so React Router's default error screen replaces the app |
| Status | Open |
| Proposed phase | 1C (roadmap, pending approval) |

**What is wrong.** On USB Serial and Bluetooth, the Tauri COM reader forwards every newline-terminated line to React as `espState`. Only Wi-Fi validates with the Rust protocol types. The frontend accepts any line that starts with `{` and parses as JSON. It appends the object to `logs` before any shape check. `describeLog` assumes every non-Registration, non-System object has a `ModuleEvent` with exactly one key. Any other JSON object throws a `TypeError` inside `LogViewer`'s render, including firmware console noise, a future protocol variant, or `{"ModuleEvent":{"id":"x","event":{}}}`.

**Why this is a problem.** The failure is in render, not in a handler, so it is not caught. No `ErrorBoundary` or route `errorElement` exists anywhere in `UI/src`. The bad object stays in `logs` until the next connection, so the crash recurs every time the Logs page renders.

**How it could affect Pinora.** The operator loses the control UI, including the Disconnect button, while hardware may be active, and has to restart the application. Serial mode mixes ESP-IDF console output with protocol lines on UART0, which makes stray JSON-looking lines realistic.

**Evidence (probe output).**
```
unknown top-level object: THROWS TypeError: Cannot destructure property 'id' from null or undefined value
empty object:             THROWS TypeError: Cannot destructure property 'id' from null or undefined value
ModuleEvent with empty event: THROWS TypeError: undefined is not an object (evaluating '[kind, payload]')
```

**Plain-English explanation.** One unexpected line from the board can blank the desktop app until it is restarted.

**Recommended solution.** Validate at the receive boundary. Either `ProtocolMessageSchema.safeParse` in `InitializeListeners`, or, better, validate in Tauri for all transports exactly as the Wi-Fi path already does, so React receives typed messages only. Make `describeLog` total, with a fallback "Unrecognized message" entry. Add an `errorElement` or error boundary so a rendering fault is contained to one view.

**Risk of changing it.** Messages that are currently shown might be dropped, because the TypeScript schemas cover fewer variants than Rust (PIN-TPL-007). Validation must use the complete schema, or log unknown variants instead of discarding them.

**How we will verify the fix.** Unit tests of the receive and describe pipeline with valid, unknown-variant and malformed inputs. A component test rendering `LogViewer` with a malformed entry. A Tauri test feeding raw lines into the COM line handler.
