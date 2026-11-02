# Phase 2D — Desktop performance and structure (RESULT)

| Field | Value |
|---|---|
| Phase | 2D |
| Status | **Completed — Verified** against D-V1…D-V5. The desktop app's window was not launched (no GUI automation); store behavior is covered by characterization tests with the Tauri IPC mocked |
| Date | 2026-11-02 |
| Branch | `phase-2d/desktop`, cut from `phase-2c/runtime-sensors` @ `a6c38afe` (local, not pushed) |
| Findings | PIN-TPL-011, -014, -020 → **Resolved** |
| Firmware / wire | Unchanged |

## 1. Commits

| Step | Commit | Change |
|---|---|---|
| D1 | `80f71533` | Bounded, batched log history (5 000), cached search, export, less console IPC |
| D2 | `e2317af4` | Slow Tauri commands `async` + `spawn_blocking`; Bluetooth port cache (10 s) |
| D3 | `5d50de28` | Store split into four slices; `noImplicitAny`; store characterization tests |
| Docs | *(docs commit)* | Plan, RESULT, register |

## 2. Before and after

| Aspect | Before | After |
|---|---|---|
| Log history | Unbounded; every message copied the whole array (O(n²) per session) and re-rendered | 5 000 newest entries; one copy per 50 ms batch; Export to JSON |
| Search | Re-described and `JSON.stringify`'d every entry on every change | Summary and search text computed once per entry |
| Console → Tauri log | Every `console.*` call was an IPC round trip, including one `console.info` per message | Only warnings and errors are forwarded; no per-message logging |
| Bluetooth port query | PowerShell on Tauri's main thread, run twice per connect | Off the main thread; one query per refresh, cached 10 s for the connect check. Measured here: 0.43–0.91 s warm, about 7 s on a cold PowerShell start; a cached call takes ~0 ms |
| Connect / disconnect | On the main thread (including the 100 ms DTR/RTS pulse and the reader join) | On a blocking thread |
| Store | One 500-line object | `Modulefront.ts` (28 lines) composing `store/{transport,registry,logs,auth}.ts`; routing by type narrowing instead of casts |
| Typing | `noImplicitAny: false`; two `handleEvent(event: any)` | `noImplicitAny: true` (0 errors); no explicit `any` outside generated UI components |

## 3. Verification

| # | Check | Result |
|---|---|---|
| D-V1 | 100 000 synthetic messages | ✅ History stays ≤ 5 000 and ends at the newest message; the last 100 batches cost no more than batches 100–200 (both at capacity) |
| D-V2 | Store characterization tests | ✅ 13 tests written against the old store, passing before and after the split: connect sequences (Bluetooth, Serial), registration and event routing, re-registration keeps the store, duplicate names, schema rejection, system snapshot, bounded logs through the real listener, `deviceAuth` (a payload with a key is rejected), pairing commands, Bluetooth and USB loss, disconnect |
| D-V3 | Port query timing | ✅ §2 (ignored timing test `measure_the_windows_query`) |
| D-V4 | Suites | ✅ `bun test` **166 pass** (was 144); `tsc` clean; Vite build passes; Tauri **21 pass** + 1 ignored; clippy only the 2 baseline warnings |
| D-V5 | Mutation | ✅ No cap → 2 failures; no log push in the registry → 2 failures; no rediscovery after a Bluetooth connect → 1 failure |

## 4. Not verified

- The UI in the running desktop app (window responsiveness, React profiler timings). Responsiveness follows from the commands no longer running on the main thread, but it was not measured in the window.
- WebView2 behavior of the Export download.

## 5. Register changes

| Finding | Change |
|---|---|
| PIN-TPL-011 | Open → **Resolved** |
| PIN-TPL-014 | Open → **Resolved** |
| PIN-TPL-020 | In Progress → **Resolved** |

Totals: **2 Open · 6 In Progress · 19 Resolved** (27).

## 6. Recovery

Revert the 2D commits, or `git switch phase-2c/runtime-sensors`.

## 7. Next

Phase 2E, Build reproducibility and hardware configuration (PIN-TPL-015, -018).
