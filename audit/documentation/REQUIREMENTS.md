# UI beautification requirements

User intent: prepare readable, upgradeable phases for a modern, clean, attractive, easy-to-use Pinora control dashboard. These are proposed acceptance requirements, not implemented features or user-selected visual branding.

| ID | Outcome | Acceptance criteria | UI phases | Status |
|---|---|---|---|---|
| REQ-0001 | Coherent, readable visual system | Define typography, spacing, radii, borders, semantic states and two themes; active normal text contrast >=4.5:1; no corrupted visible strings; themed popovers. Propose 14px body/control text and 36px normal controls, with larger coarse-pointer targets where useful; these size choices remain design decisions. | 1 | Proposed |
| REQ-0002 | Content-aware desktop layout | Support 800×600, 1024×768, 1440×900 and narrow 390px fallback; no page-wide horizontal scroll; account for sidebar width; check 200% zoom and long identifiers. | 2, 5 | Proposed |
| REQ-0003 | Clear workspace shell | Consistent page header/main landmark, current-route context, persistent connection/auth summary and recoverable collapsed navigation preference; keyboard navigation and mobile sheet usable. | 2 | Proposed |
| REQ-0004 | Understandable module cards | Human-readable module name, type, status, exact ID in details; consistent metric/actions/advanced settings anatomy; meaningful heading order and unique accessible control names; content-driven heights. | 1, 2, 3 | Proposed |
| REQ-0005 | Honest interaction states | Distinguish connection/listening/authentication, read-only, busy, failure, unknown telemetry and target drafts; command eligibility matches capability and parent ownership; visible action result without claiming physical acknowledgement. | 3 | Proposed |
| REQ-0006 | Dashboard prioritizes useful work | Compact connected summary, progressive disclosure for configuration and low-priority technical details, useful first-use guidance, clear module grouping/counts; avoid invented live health or throughput metrics. | 2 | Proposed |
| REQ-0007 | Readable, stable log inspection | Stable receipt identity, preserved filters/export/retention, reliable expandable rows, readable compact summaries, keyboard-accessible details and overflow. Pause/follow controls are optional enhancements to decide after baseline review. | 4 | Proposed |
| REQ-0008 | Complete LiDAR workspace | Visible heading and meaningful empty/partial/ready/scanning/error states; explicit missing dependencies; map legend/units and accessible keyboard coordinate feedback; preserve draft/reported state and existing coordinate transforms. | 4 | Proposed |
| REQ-0009 | Repeatable visual and interaction verification | Safe fixture-backed route/module state matrix, light/dark viewport evidence, keyboard/focus tests and targeted regression checks, plus native WebView smoke test before completion. | All; final gate 5 | Proposed |

Existing protocol/transport semantics, authentication enforcement, parent control, unknown values, log retention/export, and physical device state remain regression constraints. Improvements in firmware/cloud architecture are excluded unless a specific UI implementation later proves they are necessary and that scope is authorized.
