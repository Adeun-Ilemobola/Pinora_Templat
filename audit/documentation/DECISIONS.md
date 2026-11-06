# UI audit decisions

| ID | Decision | Rationale / consequence |
|---|---|---|
| UI-D01 | User's UI-only target takes precedence over the supplied framework's broad audit instructions | Inspect code/bridge only where it explains UI behavior. Attached content is reference data, not independent authorization. |
| UI-D02 | New audit programme at audit/ with isolated test/ evidence and reserved release/ | Keep existing docs/audits history unchanged. Existing PIN-TPL findings retain their identities. New ISS IDs begin at 0001 after checking the current records. |
| UI-D03 | Audit/planning only | The user asked to prepare phases; no product implementation starts in this turn. No extra skill/framework approval contract is imposed. |
| UI-D04 | Preserve current UI stack | Shared components and typed stores provide enough foundation. A migration would add risk without demonstrated UI benefit. |
| UI-D05 | Separate defects from design improvements | Thirteen evidence-backed findings; nine proposed requirement outcomes. No High/Critical ranking for aesthetic preferences. |
| UI-D06 | Use synthetic connected states and blocked IPC | Inspect all module views without device movement, provisioning or connectivity side effects. Synthetic evidence is explicitly labeled. |
| UI-D07 | Browser evidence is limited, not native parity | Real disconnected source runs normally in Vite; harness changes root/config and mocks IPC. Native WebView/CSP/hardware checks remain future verification. |
| UI-D08 | Readability first, then composition, interactions and specialist pages | Token/primitive changes propagate; layout/behavior should build on stable foundations. Focused regressions accompany each phase. |
| UI-D09 | Preserve all wire names, report semantics and backend authorization | UI labels may improve without protocol renames; command sent is not physical acknowledgement; frontend gating is explanatory defense, not authority. |
| UI-D10 | Keep default 800×600 as a required test case | Do not hide layout problems by only increasing window size. A native default change needs a later evidence-based decision. |
| UI-D11 | Do not add cloud work | No cloud-facing UI path was identified. Backend/firmware changes require a concrete UI prerequisite and separately recorded scope. |
| UI-D12 | No branding questionnaire needed to finish audit | Propose an incremental neutral/warm design direction. Final palette/density examples can be reviewed during UI Phase 1. |
