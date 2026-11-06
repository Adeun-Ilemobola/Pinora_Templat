# UI audit validation baseline

Date: 2026-11-05. Environment: Windows/PowerShell, Bun 1.4.0, Node v24.19.0, installed UI dependencies. Source revision `127dd90e0428ea0d90dfd77e48712e639d386d15`. Installed Vite reported 8.3.0; the package manifest specifies a compatible range. No install or lockfile update was run.

## Executed checks

| Check | Command (working directory) | Result | Evidence |
|---|---|---|---|
| Existing frontend suite | `bun test` (`UI`) | 173 pass, 0 fail; 1,254 expect calls; 5 files | [frontend-tests.txt](../../test/runs/ui-audit-2026-11-05/frontend-tests.txt) |
| TypeScript | `bun x --no-install tsc --noEmit` (`UI`) | Exit 0; no diagnostics | [typecheck.txt](../../test/runs/ui-audit-2026-11-05/typecheck.txt), intentionally empty |
| Production frontend | `bun x --no-install vite build --outDir ../test/build/ui-audit-2026-11-05` (`UI`) | Exit 0 | [frontend-build.txt](../../test/runs/ui-audit-2026-11-05/frontend-build.txt) |
| Sample token contrast | `python test/runs/ui-audit-2026-11-05/measure-contrast.py` (root) | Light muted normal-text target fails; other sampled pairs pass | [contrast-results.json](../../test/runs/ui-audit-2026-11-05/contrast-results.json) |
| Browser review | Chrome at explicit viewports; unmodified frontend and isolated fixture | Completed limited cases below | [observations](../../test/runs/ui-audit-2026-11-05/browser-observations.json), screenshots |
| Documentation consistency | Local Markdown target and issue-record check | 32 Markdown records; no broken local links; counts/status/confidence reconciled | [documentation-check.txt](../../test/runs/ui-audit-2026-11-05/documentation-check.txt) |

Build notes: generated main JavaScript chunk 751.52kB (238.63kB gzip); CSS 96.96kB (17.85kB gzip). Warning: a chunk exceeds 500kB. Vite also warned about future native config-loader compatibility for `__dirname`. Both are non-blocking diagnostics; no launch-performance measurement or dependency defect is inferred.

## Browser observations performed

- Unmodified disconnected Dashboard at 800×600 in dark/light and 1440×900 light; measured card positions and control heights.
- Unmodified light-theme Transport popup: forced dark class and dark computed background.
- Sidebar collapse/reload: state returns to expanded.
- Synthetic connected Dashboard with all eight standalone module families, unknown and selected reported values; LiDAR composite with child modules.
- Synthetic NoKey/Bluetooth state: security card says Read-only; Toggle LED, Start ranging, Read mode remain enabled.
- LED generated range input: no aria-label/aria-labelledby; PivotSlider's thumb has a label.
- Synthetic LiDAR missing child composition: shell remains but no page body/header.
- Populated synthetic logs (30 events): search `event 1:` yields 1 of 30; Show details expands the correct JSON.
- Narrow Logs at 390×844: document scrollWidth 375px, no page horizontal overflow in that state; toolbar wraps and nested log scroller is present.
- Theme and viewport overrides were temporary; original preview returned to dark and viewport override reset before handoff.

## Reproduction

Unmodified frontend from `UI/`: `bun run dev --host 127.0.0.1 --port 1422`. Open localhost URL. Native IPC is absent in an ordinary browser; its startup error must not be reported as a packaged-app bug.

Synthetic evidence harness from root:
```powershell
bun UI/node_modules/vite/bin/vite.js --config test/runs/ui-audit-2026-11-05/preview.config.mjs
```
Open `http://127.0.0.1:1423/test/runs/ui-audit-2026-11-05/preview.html`. Routes use `#/logs` and `#/lidar`; query `?readonly` supplies NoKey/Bluetooth; `?missingChild#/lidar` supplies partial composition. The fixture replaces Tauri core invoke with an audit-only stub. It returns synthetic port/status data and rejects other commands; no physical transports or key actions are opened.

The harness uses a separate Vite root and cache. It is an inspected UI evidence aid, not a production-parity guarantee or a permanent automated regression suite. The baseline image evidence may precede later additions of sample values to the same fixture; exact visual pixel reproduction from the final harness is not guaranteed.

## Not executed / no claim made

Native Tauri app launch/build/package, physical device commands, pairing/unpairing, real sensor/scan states, real screen-reader session, full keyboard walkthrough, reduced-motion exercise, OS theme change, 200% zoom, complete 1024px matrix, log rollover reproduction, native CSP and performance measurement. The existing suites mock IPC and do not validate rendered components.

This is a successful diagnostic frontend baseline, not a completed beautification or release-validation result.
