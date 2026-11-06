# UI audit cleanup and retention

Date: 2026-11-05.

| Artifact | State / reason |
|---|---|
| audit/ | Retained durable findings, roadmap and handoff |
| test/runs/ui-audit-2026-11-05/*.png | Retained before screenshots; all synthetic files labeled in filename/index |
| test/runs/ui-audit-2026-11-05/*.txt and *.json | Retained test/build/measurement/browser observations |
| preview.html / preview.tsx / preview.config.mjs / mock-core.ts | Retained reproducible isolated fixture; no product entry-point changes |
| measure-contrast.py | Retained reproducible token calculation |
| test/build/ui-audit-2026-11-05/ | Retained disposable diagnostic frontend build; not a release. May be removed later after confirming evidence retention needs |
| test/.gitignore | Added narrowly scoped ignores for diagnostic build outputs and per-run Vite caches; evidence and fixture sources remain visible to Git |
| test/runs/ui-audit-2026-11-05/vite-cache/ | Disposable fixture dependency cache; retained for repeat inspection |
| UI/node_modules/.vite/ | Existing dependency cache may have been updated by the original Vite preview; unavoidable tool convention, ignored by Git. No dependency install performed |
| UI/vite.config.ts.timestamp-* | Vite created a transient config import wrapper during diagnostics; ignored *.tmp, and it may disappear on normal tool shutdown. No product configuration was edited |
| release/ | Reserved, no candidate output |
| Preview processes | Audit-created servers on 1420, 1422 and 1423 stopped via their tool sessions at handoff |
| Browser | Temporary viewport override reset; original preview theme restored to dark; audit-created tab left ephemeral for automatic cleanup |

No pre-existing file, previous release, log history, key, firmware edit, or dependency directory was deleted. No repository-wide cleanup/reset. Placeholder README files retain intentionally empty workspace areas. Generated build/cache files remain disposable and are not required in a future commit; retain the reports/screenshots/harness that support findings.
