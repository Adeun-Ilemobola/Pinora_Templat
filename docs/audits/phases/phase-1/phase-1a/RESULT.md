# Phase 1A — Firmware host-test harness and CI (RESULT)

| Field | Value |
|---|---|
| Phase | 1A |
| Status | **Completed — Verified** (A1–A7 pass). The CI workflow has **not run on GitHub**, because nothing is pushed; its commands were run locally |
| Approved | 2026-10-31 (roadmap + 1A, including the top-level `tests/firmware-host/` directory and CI workflow) |
| Branch | `phase-1a/host-test-harness` (local, not pushed), cut from `phase-0/structural-reorganization` @ `a2204b33` |
| Behavioral impact | None. No firmware, protocol, backend or frontend source changed |

## Commits

| Commit | Content |
|---|---|
| `6f22bd2a` | Reassessment report and roadmap committed (docs) |
| `3dc86120` | 1A plan (docs) |
| `fbc53466` | `tests/firmware-host/`: harness and 28 characterization tests |
| `8e611b3e` | `just test` / `test-protocol` / `test-firmware-host` / `test-ui` recipes; `"test": "bun test"` in `UI/package.json` |
| `3fef4539` | `.github/workflows/ci.yml`; a type alias in the protocol golden test (test code only) |

## What was added

**`tests/firmware-host/`.** This is a standalone host crate with an empty `[workspace]`, `publish = false` and a tracked `Cargo.lock`.
- **Firmware files compiled unchanged via `#[path]`:** `utilities/math.rs`, `core/module.rs`, `core/transport/emitter.rs`, `core/transport/discovery_cache.rs`. The crate mirrors the firmware module tree, so their `crate::…` paths resolve.
- **Pinned dependencies:** `uuid =1.26.1`, `log =0.4.34` and `anyhow =1.0.104`, all exactly as locked by the firmware, plus `pinora-protocol` by path.
- **Location:** outside `Firmware_Templates/`, so that directory's Xtensa `.cargo/config.toml` and `esp-1.93` toolchain file do not apply.

**Tests (28):**

| File | Tests | What they record |
|---|---|---|
| `tests/math.rs` | 10 (9 per profile) | LED 0–100 → 0–8191 mapping. **No clamp above 100, and overflow above 524 352: panic in debug, wrap to 81 in release (PIN-TPL-002).** Underflow below the input minimum. `range_i32`/`range_i16`/`range_f32` shapes. `constrain_*`. `pulse_us_to_tick` values and clamping |
| `tests/emitter.rs` | 8 | `try_emit` drops without blocking when the 128-slot queue is full and keeps the first 128. `emit_reliable` blocks until a slot frees, and errors with `Disconnected` (Display text checked). `try_emit`/`any` on a closed queue do not panic. `system_info` sends `System`. `log()` uses source id `"log"` (C-05). Clones share one queue |
| `tests/discovery_cache.rs` | 3 | Latest system snapshot kept. One registration per id, replaced in place, first-seen order. Module events ignored (PIN-TPL-012) |
| `tests/module.rs` | 7 | Fresh UUID v4 per construction. Registration shape (`lool_up_id`, `""` parent, or the parent id). Default `first_emit` sends nothing. `emit`/`log` wrap with the module id. `From<()>` → `OperationFailed`. Registry keyed by UUID, not by lookup name |

**Scripts.** `just test` runs all three suites: protocol golden, firmware host, frontend. `bun run test` is now available in `UI`. `bun.lock` is unaffected, since it does not record scripts.

**CI (`.github/workflows/ci.yml`).**
- **Rust job (ubuntu):** placeholder `wifi_config` from the example; `cargo +stable fmt --check` on all four crates; protocol test and clippy; firmware host tests (debug and release) and clippy.
- **Desktop job (windows):** `bun install --frozen-lockfile`; `bun run build` (`tsc` + Vite; it also produces the `UI/dist` that Tauri needs); `bun test`; Tauri clippy.
- **Not included:** the firmware cross-build. Lint warnings do not fail CI yet (PIN-TPL-022, Phase 3A). Permissions are `contents: read`.

## Verification

| # | Check | Result |
|---|---|---|
| A1 | `cargo test --locked` in `tests/firmware-host` | ✅ 28 pass (debug), 27 pass (release; the profile-specific overflow test swaps) |
| A2 | Mutation check | ✅ An off-by-one in `range_u32` → 2 tests fail. Emitter capacity 128 → 64 → 1 test fails. Both files restored with `git checkout`; tree clean |
| A3 | `just test` | ✅ exit 0: protocol 2, firmware host 28, frontend 16 |
| A4 | Firmware `esp-1.93` debug and release builds | ✅ pass, 11 warnings (unchanged). Release ELF text 1 113 390 / data 250 958 / bss 42 539, **identical** to the Phase 0 result |
| A5 | Protocol golden, `tsc`, `bun test`, Vite build, Tauri clippy | ✅ unchanged (golden file not modified; Tauri 2 warnings as before) |
| A6 | CI commands run locally (Windows) | ✅ fmt ×4 clean; protocol test and clippy; host tests debug and release; host clippy; `tsc`; Vite build; `bun test`; Tauri clippy. The YAML parses (2 jobs, 17 steps). **Not run:** the Ubuntu job itself, and any GitHub Actions execution |
| A7 | No change to firmware, protocol, backend or frontend sources, or to existing lockfiles | ✅ The `git diff a2204b33..HEAD` scope is limited to the 11 planned files |

### Lint notes (pre-existing, not changed in 1A)

Clippy reports 3 `redundant_field_names` warnings **inside the included firmware files** (`core/module.rs:61`, `core/transport/emitter.rs:75-76`). They are part of PIN-TPL-022 and are not fixed here, because 1A changes no firmware source. Protocol clippy with `--all-targets` reports the known `Axes::add` warning only, after the test type alias.

## Effect on findings

| Finding | Status change | Notes |
|---|---|---|
| PIN-TPL-008 | In Progress (unchanged), **scope narrowed** | Done: host harness for hardware-free firmware code, `just test`, CI workflow. Remaining: module state machines are not host-testable until the output-trait seam (1D) and ordered registry (2C) exist; dispatch becomes testable when 1B extracts `runtime/dispatch.rs`; `UI/tests` is not type-checked (it would need `@types/bun`, a lockfile change); CI has not yet executed on GitHub |

No new findings. No severity changes.

## Compatibility notes

- **New top-level directory `tests/`** (approved). External tooling that copies or scans the template's top-level folders may now see it (C-10).
- **The harness depends on the paths of four firmware files.** Moving one breaks the harness build loudly, by design.
- **CI starts running only after you push the branch.**

## Recovery

All 1A changes are additive. Revert the commits above, or `git switch phase-0/structural-reorganization`. `main` is still at `5b5c8ed4`.

## Next step

Stop for review. The next roadmap step is **Phase 1B — Firmware command safety** (PIN-TPL-001, -002, -025), which needs your authorization, plus the 1B decision on whether out-of-range LED values are clamped (recommended) or rejected.
