# Phase 3A — Low-severity cleanup and hardening (PLAN and RESULT)

| Field | Value |
|---|---|
| Phase | 3A |
| Status | **Completed — Verified**: zero warnings in every build, enforced in CI; the UART0 change verified on your ESP32. The CSP was not checked inside the running desktop window |
| Date | 2026-11-02 |
| Branch | `phase-3a/cleanup`, cut from `phase-2e/reproducible-builds` @ `08bfeec3` (local, not pushed) |
| Scope and decisions | The roadmap's scope, under your blanket approval. **Identifier kept** (`com.owner.ui`): changing it would move the app-data directory, which now holds the paired-device keys. **`.espConfig` and `pre-script.rhai` left untouched**: contract C-10 says they wait for the sibling-project audits. No TypeScript linter added: it would be a new dev dependency; `tsc` in strict mode with `noUnusedLocals` and `noImplicitAny` is the gate |
| Findings | PIN-TPL-022, -023, -024, -026 → **Resolved**; PIN-TPL-017 and -021 → **Blocked** (their remaining parts break the wire format) |

## 1. Commits

| Step | Commit | Change |
|---|---|---|
| F1–F2 | `726e499f`, `ef6a2bc3` | Warnings to zero; `-D warnings` on every clippy step in CI; CSP and devCsp; product name and title "Pinora"; `greet` removed; `start_port` returns the error message |
| F3 | `e6a62d9c` | `p.uart0` consumed by the console setup; SysLogs (source `"log"`) no longer looked up as modules; documentation drift fixed |
| — | `e9d772bd` | hardware-probe clippy |

## 2. Results

| Check | Result |
|---|---|
| Warnings | Firmware debug **0** and release **0** (was 5; 17 at the baseline). Clippy `-D warnings`: protocol, firmware host, Tauri, hardware probe all **0** |
| Suites | protocol 31, firmware host 122 (debug and release), Tauri 21 (+1 ignored timing test), frontend 167; `tsc` clean; Vite build passes; rustfmt clean on 5 crates |
| Image | Release 1 032 567 / 233 158 / 31 467, unchanged by 3A |
| On device (3A image) | `just hw-test` 23/23 and 5/5 (the USB console path is the UART0 code); loop period 1.000 ms mean, p99 ≤ 1.2 ms |
| CSP | Accepted by `tauri::generate_context!` at build time. ⏳ Not checked for violations in the running app |

## 3. Blocked findings

| Finding | Remaining part | Why blocked |
|---|---|---|
| PIN-TPL-017 | `Registration.parent_id` is `""` for top-level modules instead of `null` | Wire-breaking for every consumer (contract C-01); needs a major protocol version |
| PIN-TPL-021 | Output names `lool_up_id`, `Ckick`, `Idol`, `distant` | Wire-breaking; the corrected spellings are already accepted as input aliases (2A) |

Both should be done together in one deliberate protocol v3, after the sibling projects (`pinora_cli`, `pinora_studio`) are audited and can move with it.

## 4. Register changes

Totals: **0 Open · 0 In Progress · 2 Blocked · 25 Resolved** (27).
