# Phase 1A — Firmware host-test harness and CI (PLAN)

| Field | Value |
|---|---|
| Phase | 1A |
| Status | **Approved** 2026-10-31, together with the [roadmap](../../ROADMAP.md); this includes the new top-level `tests/firmware-host/` directory and the CI workflow |
| Branch | `phase-1a/host-test-harness`, cut from `phase-0/structural-reorganization` @ `6f22bd2a` (local, not pushed) |
| Linked findings | PIN-TPL-008 |
| Behavioral impact | **None.** Tests, scripts and CI configuration only; no firmware, protocol, backend or frontend source changes |

## Scope

1. **`tests/firmware-host/`**: a standalone host crate (`[workspace]`-isolated, `publish = false`, its own tracked `Cargo.lock`). It compiles these firmware files **unchanged** via `#[path]`:
   - `Firmware_Templates/src/utilities/math.rs`
   - `Firmware_Templates/src/core/module.rs`
   - `Firmware_Templates/src/core/transport/emitter.rs`
   - `Firmware_Templates/src/core/transport/discovery_cache.rs`

   Dependencies are pinned to the firmware's locked versions (`uuid =1.26.1`, `log =0.4.34`) plus the path dependency on `pinora-protocol`.
2. **Characterization tests** that record *current* behavior, including known defects, which are labeled with their finding IDs:
   - `math`: mapping results, boundaries, the overflow panic of `range_u32`/`range_i32` in debug (PIN-TPL-002), the widening in `range_i16`, and `pulse_us_to_tick` clamping.
   - `emitter`: lossy `try_emit` drops without blocking when the 128-slot queue is full; `emit_reliable` errors when disconnected; `log()` uses source id `"log"`; `system_info` wraps `System`.
   - `discovery_cache`: replace-in-place by module id, order preserved, module events ignored.
   - `module`: the `Module` trait defaults (registration shape, `""` parent, event wrapping, SysLog via `log`), UUID v4 identity, `From<()>`.
3. **Scripts**: `"test": "bun test"` in `UI/package.json` (the lockfile is unaffected), and `test`, `test-protocol`, `test-firmware-host` and `test-ui` recipes in the `justfile` (additive).
4. **CI**: `.github/workflows/ci.yml`. It runs format checks, protocol and host-firmware tests and clippy, Tauri clippy (Windows, after a frontend build, because Tauri needs `UI/dist`), `tsc`, `bun test` and the Vite build. The firmware cross-build is excluded. CI takes effect only if you push.

## Frozen

All contracts C-01 … C-19 stay unchanged. In particular: no change to existing lockfiles, manifests other than the `UI/package.json` scripts entry, or firmware sources.

## Verification

| # | Check | Pass criterion |
|---|---|---|
| A1 | `cargo test --locked` in `tests/firmware-host` | All pass |
| A2 | Mutation check: temporarily alter one line of an included firmware file and one test expectation | Tests fail; then the file is restored (`git diff` clean) |
| A3 | `just test` (or its three commands) | All suites pass |
| A4 | Firmware `cargo +esp-1.93 build --frozen` debug and release | Pass, with ELF sizes identical to the Phase 0 result (no firmware change) |
| A5 | Protocol golden tests, `tsc`, `bun test`, Tauri clippy | Unchanged from Phase 0 |
| A6 | CI commands | The same commands run locally where the OS allows; the YAML parses |
| A7 | Existing lockfiles and firmware sources | No diff |

## Recovery

Every change is additive. Revert the branch commits, or `git switch phase-0/structural-reorganization`.
