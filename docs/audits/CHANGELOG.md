# Audit and modernization changelog — Pinora Template

Newest entries first. Each entry is tagged **[AUDIT]** (documentation only, no source change) or **[SOURCE]** (template source, build or config changed).

## 2026-10-31 — Initial audit **[AUDIT]**

- Created the `docs/audits/` tree: README, architecture (4 documents), findings register (26 findings: 0 Critical, 6 High, 14 Medium, 6 Low), phase index, Phase 0 plan, initial-audit report, validation baseline and probe sources.
- Ran hardware-independent checks with build output isolated outside the repository. Protocol and Tauri `check`/`clippy` pass; `tsc --noEmit` passes; `cargo fmt --check` fails (pre-existing). Firmware build not run (needs approval D5).
- Ran two throwaway probes outside the repository against real sources (protocol wire shapes and LED arithmetic; desktop log helpers).
- Mirrored all of the above to `C:\Dev\Pinora\Audits\Pinora_Templat\` and created `C:\Dev\Pinora\Audits\README.md`.
- **Source changes: none.** The template working tree is unchanged apart from the new, untracked `docs/audits/` directory. Nothing was committed.
- Existing `docs/architecture-audit.md` was left untouched; the new tree sits alongside it.
