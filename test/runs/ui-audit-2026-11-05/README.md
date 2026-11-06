# UI audit evidence — 2026-11-05

Source: main @ 127dd90e0428ea0d90dfd77e48712e639d386d15. Product source unchanged. No device operation performed.

| Evidence | Meaning |
|---|---|
| dashboard-dark-800x600.png | Unmodified disconnected dashboard at configured desktop viewport; includes browser-only missing IPC error |
| dashboard-light-800x600.png | Same in light theme |
| dashboard-light-1440x900.png | Wide unmodified disconnected dashboard; empty overview height |
| select-light-theme.png | Unmodified light document with forced-dark select popup |
| synthetic-dashboard-dark-1440x900.png | Actual dashboard views with synthetic connected/unknown module state and IPC stub |
| synthetic-modules-dark-1440x900.png | Unequal module content height and visibly corrupted units; scroll position is intentional |
| synthetic-readonly-controls.png | Synthetic NoKey/Bluetooth overview; DOM observations establish write control enabled status |
| synthetic-lidar-missing-children.png | LiDAR exists but dependencies missing; route body empty |
| synthetic-lidar-dark-1440x900.png | Full composite with synthetic children; no real sensor samples |
| synthetic-logs-dark-390x844.png | Populated synthetic history, narrow layout |
| synthetic-log-filter-details.png | Search returns 1 of 30 and details expand correctly |
| browser-observations.json | Selected measured layout, semantics and interaction results |
| frontend-tests.txt | 173 passed, 0 failed |
| typecheck.txt | Empty output with recorded exit 0 |
| frontend-build.txt | Successful isolated build and non-blocking diagnostics |
| documentation-check.txt | Local link and issue count/status consistency check |
| measure-contrast.py / contrast-results.json | Reproducible sampled token luminance/contrast calculation |

Preview reproduction: see [TEST_REPORT.md](../../../audit/documentation/TEST_REPORT.md). Harness files are local audit fixtures, not app source or production tests; all action IPC is blocked. Later fixture additions of telemetry/log samples may differ from early screenshots. Temporary Vite servers are stopped at handoff.

Generated caches/build outputs are disposable; durable evidence/harness should be retained. Do not commit the entire dependency cache or diagnostic build by accident.
