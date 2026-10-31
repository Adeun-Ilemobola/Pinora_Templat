# Validation baseline — initial audit (2026-10-31)

All checks are hardware-independent. **No device was flashed, connected, powered or commanded. No Bluetooth or Wi-Fi radio activity was initiated.**

To keep the repository unchanged, every Rust build used an isolated `--target-dir` outside the repository, and TypeScript ran with `--noEmit`. Formatting was `--check` only.

## Environment

| Item | Value |
|---|---|
| OS | Windows 11 Home 10.0.26300 |
| Host Rust | `cargo 1.97.1` (stable-x86_64-pc-windows-msvc) |
| ESP toolchains installed | `esp-1.93` (project-selected), `esp-1.97`, `esp-1.98.1`, `esp-1.99` |
| Bun | 1.4.0 |
| TypeScript (project-local) | 6.0.3 |
| Commit | `5b5c8ed4` on `main`, clean tree |
| Local untracked config | `protocol/src/wifi_config.rs` exists (needed to compile); its contents were **not read** |

## Static checks

| # | Check | Command (run from package directory) | Result |
|---|---|---|---|
| S1 | Protocol compile | `cargo +stable check --locked --target-dir <scratch>` | ✅ pass (5.3 s) |
| S2 | Protocol lint | `cargo +stable clippy --locked --target-dir <scratch>` | ✅ pass, **1 warning**: `method add can be confused for std::ops::Add::add` (`protocol/src/module/imu/imu_type.rs`) |
| S3 | Tauri backend compile and lint | `cargo +stable clippy --locked --target-dir <scratch>` in `UI/src-tauri` | ✅ pass (41.8 s), **2 warnings**: `variant Disconnected is never constructed`; `fields connection_type, transport_type, error are never read` (`lib.rs`) |
| S4 | Frontend type check | `./node_modules/.bin/tsc --noEmit -p tsconfig.json` in `UI` | ✅ pass (9.4 s). Note: `tsc` resolved the tracked `Modulefront.TS` as `Modulefront.ts` (case-insensitive file system; PIN-TPL-015) |
| S5 | Rust formatting | `cargo +stable fmt --check` | ❌ diffs: firmware **78** hunks, protocol **21**, Tauri **6** (PIN-TPL-022) |
| S6 | Lockfile integrity | `sha256sum protocol/Cargo.lock` before and after | ✅ unchanged (`a5a320b5…ccd1dbb3e`) |
| S7 | Working tree | `git status --porcelain` after all checks | ✅ clean; no new ignored outputs; `UI/src-tauri/gen` not modified |

## Not run (and why)

| Check | Reason | Plan |
|---|---|---|
| Firmware `cargo +esp-1.93 check/build --frozen` | Writes into the shared firmware cache `C:/t` and `Firmware_Templates/.embuild` (outside the permitted documentation locations). A fresh isolated target dir would rebuild ESP-IDF, and the README notes that long Windows paths fail | Run as the **Phase 0 baseline** once authorized to use `C:/t` (or an approved short path). Record debug and release binary size for comparison |
| Vite production build (`bun run build`) | Writes `UI/dist` inside the repository | Phase 0 baseline with `--outDir` pointing at a scratch directory |
| Any test suite | None exists (PIN-TPL-008) | Phase 0 adds characterization tests |
| Hardware, radio and end-to-end checks | Not authorized | Listed per finding under "How we will verify the fix" |

## Dynamic probes

Two throwaway probes were built **outside the repository**. Each used the template's real source by path and modified nothing.

### P1 — Protocol wire shapes and LED arithmetic (Rust)

The probe crate depends on `protocol/` by path and includes `Firmware_Templates/src/utilities/math.rs` verbatim via `#[path]`.

```
LED SetState: {"id":"u","command":{"Led":{"SetState":{"state":80}}}}
LED Toggle: {"id":"u","command":{"Led":{"Toggle":{}}}}
Rangefinder StartRanging: {"Rangefinder":"StartRanging"}
Lidar StartScan: {"Lidar":{"StartScan":{}}}
IMU Mode event: {"Imu":{"event_type":"Mode","mode":"Idle"}}
SysLog event: {"SysLog":{"text":"t","raw_err":null,"priority":"Low"}}
Registration: {"Registration":{"id":"u","module_type":"Led","lool_up_id":"led1","parent_id":""}}
LED state=4e9 deserializes: true
```

`range_u32(state, 0, 100, 0, 8191)` (the LED mapping with a 13-bit max duty):

| state | debug build | release build |
|---|---|---|
| 100 | 8191 | 8191 |
| 101 | 8272 | 8272 |
| 524 352 | 42 949 672 | 42 949 672 |
| 524 353 | **panic: attempt to multiply with overflow** | **81** (wrapped) |
| 4 000 000 000 | **panic** | 19 894 661 |

`esp-idf-hal` `LedcDriver::set_duty_with_hpoint` clamps duty to `max_duty` (verified in the locked revision `846cff17`), so values ≥ 8191 produce full brightness, and wrapped values such as 81 produce ≈1 %. This supports PIN-TPL-002 and contracts C-01 / C-13.

### P2 — Desktop log helpers on malformed input (TypeScript, Bun)

The probe imports `UI/src/lib/logs.ts` directly.

```
valid LED event: OK -> Module events
unknown top-level object: THROWS TypeError: Cannot destructure property 'id' from null or undefined value
empty object: THROWS TypeError: Cannot destructure property 'id' from null or undefined value
ModuleEvent with empty event: THROWS TypeError: undefined is not an object (evaluating '[kind, payload]')
```

This supports PIN-TPL-006.

## Dependency source verification

Behavior claims about dependencies were checked against the exact revisions locked in `Firmware_Templates/Cargo.lock`, using the local Cargo Git cache:

| Claim | Source checked |
|---|---|
| Returning from Rust `main` returns from `app_main` (no reboot) | `esp-idf-sys@33045844` `src/start.rs` |
| `LedcDriver` clamps duty; `Drop` calls `ledc_stop` | `esp-idf-hal@846cff17` `src/ledc.rs` |

## Baseline summary for Phase 0 comparison

| Check | Baseline |
|---|---|
| Protocol check / clippy | pass / 1 warning |
| Tauri clippy | pass / 2 warnings |
| `tsc --noEmit` | pass / 0 errors |
| `cargo fmt --check` | fail (78 / 21 / 6 hunks) |
| Firmware build | **not yet captured**; required before Phase 0 changes |
| Vite build | **not yet captured**; required before Phase 0 changes |
| Tests | none exist |
