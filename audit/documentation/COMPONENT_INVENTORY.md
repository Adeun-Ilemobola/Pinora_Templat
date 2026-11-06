# UI component inventory and coverage

Review basis: 2026-11-05. “Rendered” means browser preview; synthetic states are not hardware validation.

| Surface | Current composition | Audit disposition |
|---|---|---|
| App/main | Hash router, theme provider, console warning/error forwarding | Preserve routing and error isolation; improve theme resolution |
| AppLayout | Persistent 16rem sidebar, 64px header, 7xl content, theme toggle | Content-aware compact shell; route/main semantics; persistent collapse |
| Dashboard | Transport + stats, security, registry switch/grid | Move connected status ahead of setup details; improve module grouping/identity and unequal row heights |
| TransportForm | Three modes, COM/rate/port inputs, refresh, busy/error | Keep real transport semantics; compact summary/config disclosure; improve user-facing recovery copy without hiding diagnostic detail |
| Esp32StatsCard | Snapshot fields, disconnected/waiting state, warnings | Clearly label snapshot; avoid empty stretched card; format existing fields without fabricating live health |
| DeviceSecurityCard | Auth summary and USB pair/key actions | Preserve key/backend boundary; align action availability and pending state |
| ModuleCard | Type, full UUID, body | Add readable name/type/status/details contract and semantic headings |
| ParentControlledState | Shared note with badge/description | Preserve constraint, reduce repetition when nested |
| LED | Reported brightness, local slider draft, toggle | Input accessible name; promise/result feedback; distinct readable names |
| Button | Physical input activity and event count | Preserve event-only semantics; no invented pressed/released state |
| Servo | Reported angle and direct-change pivot slider | Repair symbols; define preview/commit behavior; feedback/unknown presentation |
| Stepper | Multi-turn dial, numeric draft, reported metrics, advanced details | Preserve turn arithmetic and keyboard gestures; normalize density and state feedback |
| IMU | Scaled vectors, raw counts disclosure | Preserve units/unknowns; no invented orientation/3D model |
| Rangefinder | Distance, mode/budget, start/stop and drafts | Reuse common action/state anatomy and useful advanced disclosure |
| Remote receiver | Last key/activity pulse/count | Preserve event-only meaning; repair separator |
| RFID | Last tag, write result, read/write modes, payload validation | Repair copy; standardize pending/error/result and capability gating |
| LiDAR page | Lookup resolution + composite with fixed child names | Keep header for empty/partial composition; dependency explanation |
| LiDAR controls/map | Canvas, ROI, step/scan, motor controls, range | Preserve coordinates/drafts; improve legend/copy/keyboard feedback; inspect actual samples later |
| Logs | Filter toolbar, bounded virtual rows, details/export/clear | Stable receipt identity; readable density and viewport fit; preserve filtering and export |
| RouteError | Alert + clear-history/dashboard actions | Preserve isolation; polish diagnostic disclosure as part of state patterns |
| PivotSlider | Radix controlled slider with neutral pivot | Already named thumb and keyboard foundation; retain bounded value math |
| StepperDial | Pointer capture, cancel, keyboard, multi-turn target preview | Preserve interaction model and strong reported/target distinction |
| Primitive layer | Card/button/input/select/slider/switch/alert/sidebar plus support components | Readable defaults; theme-correct portals; semantics; avoid page-local size overrides |

Detailed source coverage: all nine module view files and domain components above read; shared primitives listed in audit report read in detail; remaining support primitives sampled. External libraries, unused asset files, all primitive permutations, and the rest of the firmware/backend were not comprehensively audited.
