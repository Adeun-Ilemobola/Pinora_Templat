# UI beautification delivery plan

Programme: **UI**. These phase numbers are independent of the historical firmware/engineering phases under `docs/audits`. UI Phase 0 is complete. UI Phases 1–5 are proposed and have not started.

The user's request authorizes audit and phase preparation. Start product implementation only after a follow-up instruction names the intended phase. This boundary follows the user's requested scope; it is not an approval demand inferred from the attached reference. No deployment, hardware writes, key changes, dependency installation, commits or releases are part of this plan unless separately requested.

## Sequence

| UI phase | Outcome | Findings | Dependencies |
|---|---|---|---|
| 0 | Evidence-backed audit, inventory, visual direction and roadmap | All identified | Complete |
| 1 | Readable visual foundations and named controls | ISS-0001, 0007, 0009, 0010, 0012 | Phase 0 |
| 2 | Useful shell/dashboard hierarchy and readable module identity | ISS-0002, 0011, 0013 | Phase 1 |
| 3 | Consistent module action eligibility and feedback | ISS-0003, 0004 | Phases 1–2 |
| 4 | Complete LiDAR states and stable log inspection | ISS-0005, 0006 | Phases 1–3 |
| 5 | Cross-app visual/native/accessibility completion evidence | ISS-0008 plus verification of all | Phases 1–4 |

Each phase is a standalone reviewable outcome. No lettered subphases are needed. Keep the product buildable after every phase. A phase may add targeted tests needed for its fixes; final verification is not postponed wholesale to Phase 5.

## UI Phase 0 — audit and preparation (complete)

Outcome: source and rendered browser review; 13 issue records; nine proposed requirements; current/proposed UI architecture; before screenshots and baseline validation; phased scope.
Excluded: product modifications, hardware activity, source/version/dependency changes.
Evidence: [audit report](AUDIT_REPORT.md), [test report](TEST_REPORT.md), [browser run](../../test/runs/ui-audit-2026-11-05/README.md).
Gate: records reconciled, no product diff introduced, limitations disclosed. Next scope: UI Phase 1.

## Shared phase execution and verification rules

- Before editing, inspect `git status --short`, this index and any newer instructions. Preserve the existing firmware changes and all unrelated work.
- Use `test/runs/<new-unique-ui-run>/` for screenshots/logs and `test/build/<run>/` for diagnostic output. Retain the original baseline.
- Run from `UI/`: `bun test`, `bun x --no-install tsc --noEmit`, and `bun x --no-install vite build --outDir ../test/build/<run>`. The last output path must be a new directory; do not empty unrelated folders.
- Use hardware-free fixtures for failures/delays/read-only/parent-controlled states. Do not connect or move hardware just to get a screenshot.
- Compare relevant routes at 800×600 and 1440×900 in both themes; include 1024×768, 390×844 and 200% zoom for layout changes. Verify control visibility, no document horizontal overflow, meaningful accessible names, focus order and keyboard operation. A screenshot alone does not validate interaction.
- Reproduce the individual issue's acceptance cases; source review alone cannot close a runtime-sensitive finding.
- Keep original wire names, UUID routing, null telemetry, parent restrictions, log payload/export and geometry contracts. Frontend capability checks supplement backend enforcement.
- Update master README, issue index/files, requirements, repair evidence, progress, tests, cleanup and handoff. Keep `0.8.0-alpha` until a real delivery/version decision is requested; no RC is implied.
- Recovery: revert only that phase's recorded patch/commit, never a repository-wide reset; preserve user edits and evidence. If a shared token or helper causes a broad regression, restore that layer and narrow the change.
- At phase completion, report exactly what passed/unverified, provide before/after evidence and the next concrete scope. Further stages remain unstarted until requested.

## UI Phase 1 — visual foundations and readability

**Objective/outcome:** establish a consistent, readable foundation that existing pages inherit without a structural rewrite.
**Requirements/issues:** REQ-0001, REQ-0004 (control naming), REQ-0009; ISS-0001, 0007, 0009, 0010, 0012.
**Affected:** `UI/src/index.css`, `theme-provider.tsx`, `components/ui/{card,button,input,select,slider,...}`, affected view strings and LED slider labeling; focused rendered fixtures/tests.

**Ordered work:**
1. Confirm the baseline state catalog and choose typography/control density with representative dashboard/module/log samples. Start from existing neutral/warm palette and three semantic font roles.
2. Correct active text contrast and document semantic status/focus tokens; tune consistent readable type, spacing, heights and radii. Audit disabled styling separately.
3. Remove the forced popup theme; model stored/resolved/system theme consistently with listener cleanup and a correctly labeled toggle.
4. Repair corrupted UI literals and use display labels for raw wire spellings; do not rename wire/schema values.
5. Name the actual LED range input and verify multiple instances. Normalize primitive naming APIs only as far as needed.
6. Capture before/after foundation states and add focused regressions for contrast/token data, theme behavior and accessible input naming.

**Exclusions:** dashboard hierarchy replacement, new navigation features, actuator command cadence, cloud/firmware changes and log retention redesign.
**Dependencies/assumptions:** existing dependencies installed; use current primitive APIs. New test tooling/dependencies need a specific justified choice; no framework migration.
**Risks/controls:** token defaults affect all consumers; inspect portal, focus, disabled, warning/error and canvas colors across routes. Remove font imports only after confirming no reference.
**Acceptance:** targeted active normal text pairs meet 4.5:1; themes and selects agree; no known malformed symbols; each LED slider has a unique meaningful name; system preference mocks update correctly; shared checks pass.
**Evidence/deliverables:** token/type/density notes, targeted tests, two-theme screenshots, updated issue/repair records. No source version/RC change.
**Completion gate:** ISS-0001/0007/0009/0010/0012 Verified only where their stated validation passes. Next requested scope: UI Phase 2.

## UI Phase 2 — shell, dashboard hierarchy and module identity

**Objective/outcome:** a useful default-size workspace whose layout makes device state and module work easy to find.
**Requirements/issues:** REQ-0002, REQ-0003, REQ-0004, REQ-0006, REQ-0009; ISS-0002, 0011, 0013.
**Affected:** AppLayout, Dashboard, TransportForm, Esp32StatsCard, DeviceSecurityCard, ModuleCard, shared page/state patterns and module header consumers, sidebar preference initialization.

**Ordered work:**
1. Define PageHeader/SectionHeader and stable main/skip-navigation structure; show current route plus compact shared connection/auth context.
2. Adapt sidebar/content breakpoints to useful remaining width, restore navigation preference, retain keyboard shortcut and narrow-screen overlay.
3. Separate connection setup from connected summary. Keep connection/auth errors visible; disclose infrequent configuration/system details without hiding warnings.
4. Show lookup names as module identity with type/status; put exact UUID in accessible details/copy presentation. Handle long/missing names and duplicate-name warnings.
5. Replace accidental equal-height stretching with deliberate content-sized card layout and useful grouping; introduce shared empty/loading/error recipes only where reused.
6. Compare viewport/zoom states and test navigation/preference/name wrapping.

**Exclusions:** new module capabilities, favorites/search unless separately justified, invented telemetry, default native window-size change without a recorded reason, command semantics changes.
**Dependencies/risks:** Phase 1 primitives stable. Connection forms must remain usable throughout mode switches and error recovery; keep one state owner.
**Acceptance:** 800×600 use no longer requires navigating past large empty setup panels for basic connected work; no page-wide overflow at required sizes/zoom; module names identify at least three LEDs; collapse state survives reload; route heading/main/focus remain useful.
**Evidence/deliverables:** before/after dashboard/shell matrix, navigation and name cases, updated architecture/records. Shared checks pass.
**Recovery:** preserve transport state contract; restore shell/dashboard patch independently if navigation or connection presentation regresses.
**Completion gate:** assigned issue tests pass and records match; native sizing still marked unverified unless actually exercised. Next requested scope: UI Phase 3.

## UI Phase 3 — module actions and feedback

**Objective/outcome:** every control communicates what is editable, what is being sent, what failed, and what the device actually reported.
**Requirements/issues:** REQ-0004, REQ-0005, REQ-0009; ISS-0003, 0004.
**Affected:** auth/capability presentation, LED/servo promise returns, relevant module views, command feedback helper and DeviceSecurityCard. Other stores only where needed to align results.

**Ordered work:**
1. Specify/test a capability table for USB, authenticated, pending, NoKey, rejected/failed, disconnected, parent ownership and busy.
2. Apply eligibility and explanatory disabled reasons consistently in views/handlers, preserving backend authorization.
3. Return LED/servo action promises; present pending/result/error locally and prevent duplicate in-flight submissions where appropriate.
4. Document per-actuator preview/commit policy before changing any send cadence; preserve reported telemetry separate from drafts and sending state.
5. Bring metric/primary action/advanced settings anatomy into alignment across all nine families; preserve event-only physical input meanings.
6. Test delayed success, rejection, reconnect, stale drafts and missing telemetry with blocked IPC.

**Exclusions:** firmware motion planning, bounds/protocol changes, device-key provisioning during tests, new hardware support.
**Risks/controls:** motion controls are sensitive to accidental repeated commands; fixture tests verify emitted command count, gesture cancellation and reported/target distinction. Serial behavior must not accidentally become auth-blocked.
**Acceptance:** read-only connections offer no enabled write affordance; failures visible without console access; no discarded rejected action promises; busy state and draft behavior clear; all existing store/protocol tests still pass.
**Evidence/deliverables:** capability matrix, focused command regressions, module before/after states, repair traceability.
**Recovery:** isolate capability selector and action-return changes; revert phase patch rather than weakening backend enforcement.
**Completion gate:** assigned issues Verified with actual tests; hardware behavior remains unverified unless separately exercised. Next requested scope: UI Phase 4.

## UI Phase 4 — logs and LiDAR workspaces

**Objective/outcome:** reliable diagnostic inspection and a complete, understandable LiDAR page.
**Requirements/issues:** REQ-0007, REQ-0008, REQ-0009; ISS-0005, 0006.
**Affected:** LogViewer/log receipt metadata and tests; LidarPage, composite view and presentational children. Geometry/store changes only if necessary to preserve existing semantics.

**Ordered work:**
1. Reproduce the probable log rollover issue before finalizing a repair; introduce stable receipt identity that does not alter exported wire payloads.
2. Verify filtering, retained row expansion/measurement, scroll behavior and empty/error states. Tune compact readability; decide optional pause/follow only with a concrete interaction requirement.
3. Keep LiDAR header/context mounted for absent/partial registrations. Explain missing/mismatched child lookups with actionable recovery; preserve exact identifiers.
4. Refine map/ROI/scan/telemetry hierarchy, readable units and legend, selected/target distinction and keyboard coordinate feedback. Use simulated scans, never fabricated live claims.
5. Validate no-LiDAR/each-missing-child/complete/scanning/error cases and log retention/capacity cases at required viewports/themes.

**Exclusions:** protocol renames, automatic hardware discovery redesign, firmware LiDAR control changes, unrequested visualization features.
**Risks/controls:** log IDs must not change retention/export; LiDAR refactors must preserve coordinate transforms, ROI draft sync, stop access and parent restrictions. Keep geometry tests intact.
**Acceptance:** LiDAR never leaves an unexplained blank route; log expansion belongs to the same retained message across overflow/filtering; map and form controls remain usable with keyboard and narrow layout; baseline suites/build pass.
**Evidence/deliverables:** reproduction/repair evidence, populated log and LiDAR state screenshots, interaction cases and coverage updates.
**Recovery:** keep page composition and log identity changes separable; restore phase patches without clearing actual user log history.
**Completion gate:** assigned issues Verified or a documented disproven finding; no unsupported claim of real sensor validation. Next requested scope: UI Phase 5.

## UI Phase 5 — completion and native validation

**Objective/outcome:** a verified cross-page UI milestone with maintainable regression evidence.
**Requirements/issues:** REQ-0002, REQ-0009 and all prior acceptance criteria; ISS-0008.
**Affected:** durable fixture/test harness, targeted browser/component checks, CI if justified, remaining local UI polish, audit records.

**Ordered work:**
1. Promote the useful synthetic state cases to a maintained, deterministic harness; keep IPC/device actions blocked by construction and exclude test-only fixtures from product entry points.
2. Complete the viewport/theme/state/keyboard matrix, including long names, multiple LEDs, overflow/details, 200% zoom, focus visibility/order, reduced motion and real accessible names.
3. Launch the native Tauri app for disconnected navigation/theme/resize/error smoke testing without opening hardware or changing keys; compare with browser evidence and inspect runtime/CSP errors.
4. Record any remaining screen-reader/hardware-specific limits; verify any required real device flows only when appropriate hardware work is separately authorized.
5. Reconcile every issue/requirement, cleanup generated artifacts after retaining useful evidence, document version/delivery status.

**Exclusions:** deployment, signing/distribution, hardware motion, key changes and release packaging without a separate request.
**Dependencies/risks:** Phases 1–4 stable; avoid brittle full-screen pixel snapshots as the only assurance. Native environment gaps are gaps, not automatic product failures.
**Acceptance:** meaningful rendered regressions enforced; shared suites/build pass; required browser matrix and native smoke checks actually run; unresolved items explicitly listed with disposition; no assertion of WCAG certification from automated checks alone.
**Evidence/deliverables:** final test report, before/after review, closed/verified issue records, handoff and cleanup inventory. Existing version standard retained; prepare an RC only if explicitly requested.
**Recovery:** no broad cleanup/reset; restore the last verified UI patch if release-blocking regression appears.
**Completion gate:** all agreed UI requirements met and material unverified checks disclosed. Next requested scope, if desired: separately scoped delivery/release candidate preparation.
