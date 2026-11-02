# Phase 2E — Build reproducibility and hardware configuration (PLAN and RESULT)

| Field | Value |
|---|---|
| Phase | 2E |
| Status | **Completed — Verified**: a fresh clone builds and passes every suite except the firmware image, which was not built from the clone (it would download ESP-IDF, about 1 GB, into it); the board passes `just hw-test` on the 2E image |
| Date | 2026-11-02 |
| Branch | `phase-2e/reproducible-builds`, cut from `phase-2d/desktop` @ `c84ea0e6` (local, not pushed) |
| Scope and decisions | The roadmap's scope (written with the work, under your blanket approval). **GPIO12 stays**: you confirmed the LEDs are wired to GPIO 12/14/27, and moving LED1 would need rewiring. **`C:/t` stays** as the firmware target directory (short Windows path, `just flash`, contract C-17); other machines override it with `CARGO_TARGET_DIR` |
| Findings | PIN-TPL-015, PIN-TPL-018 → **Resolved** |

## 1. Commits

| Step | Commit | Change |
|---|---|---|
| E1 | `8addc1ad` | `rev` pinned on the three ESP-IDF Git patches (the commits already locked); `rand` removed |
| E2–E3 | `2435464d` | `protocol/build.rs` provides `wifi_config` (local file or the example's placeholders, with a warning); CI no longer copies the example; `CARGO_TARGET_DIR` documented; `board/pin_map.rs` with host checks |
| Docs | *(docs commit)* | This file, register |

## 2. Results

| Check | Result |
|---|---|
| Lockfile (firmware) | Only intended changes: the three Git sources gain `?rev=` (same commits); `rand`, `rand_core`, `chacha20`, `cpufeatures 0.3.1` removed. `--frozen` builds pass |
| Image | Release ELF **unchanged**: 1 032 567 / 233 158 / 31 467 |
| `embassy-time` | **Not removable.** The initial audit (and the roadmap) listed it as unused. Removing it fails to link: its generic timer queue provides `__embassy_time_queue_item_from_waker`, which esp-idf-svc's `embassy-time-driver` feature needs. It is kept, with a comment in `Cargo.toml` |
| Fresh clone (scratch directory, offline, no `wifi_config.rs`) | ✅ protocol 31 tests (with the placeholder warning), firmware host 122, `bun install --frozen-lockfile` + `tsc` + 166 tests + Vite build, Tauri 21 tests. ⏳ The firmware image itself was not built from the clone (ESP-IDF download) |
| Pin map | ✅ Host tests: the pins `main.rs` takes equal `PIN_MAP`; no duplicates; strapping pins carry a note; the board docs list the LED pins |
| GPIO12 on your board | ✅ The board booted normally on every reset and power-on in Phases 2B–2E (more than 15 boots, several by DTR/RTS reset): the LED circuit does not hold GPIO12 high at reset |
| On device (2E image) | ✅ `just hw-test`: 23/23 and 5/5 |
| Suites in the main checkout | ✅ protocol 31, host 122, Tauri 21, frontend 166; firmware release builds (5 warnings) |

## 3. Register changes

| Finding | Change |
|---|---|
| PIN-TPL-015 | In Progress → **Resolved** (fresh clones build; Git patches pinned; unused `rand` removed; `C:/t` kept with an override). Residual: the firmware still needs the ESP-IDF toolchain downloaded on first build |
| PIN-TPL-018 | In Progress → **Resolved** (pin map with checks; GPIO12 kept by your wiring, verified booting on your board) |

Totals: **2 Open · 4 In Progress · 21 Resolved** (27).

## 4. Recovery

Revert the 2E commits, or `git switch phase-2d/desktop`. The lockfile is restored from Git.

## 5. Next

Phase 3A, Low-severity cleanup and hardening.
