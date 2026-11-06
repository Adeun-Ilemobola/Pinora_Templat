# Proposed visual direction — a calm hardware workspace

This is a design recommendation for the next phases, not an implemented or approved visual redesign.

## Visual principles

- Put the device and useful controls first. Connection/auth state should be legible across routes; full connection setup belongs in an expandable configuration area after connection.
- Use restrained neutral surfaces and one warm accent as a starting point, building on the existing palette. Use semantic success/warning/error colors consistently and pair them with words/icons.
- Increase normal body/control readability before adjusting decorative elements. Propose a 14px normal text baseline, 12px supporting labels, readable tabular metrics, and one body/one heading/one mono family. Current Geist/Raleway imports have no assigned semantic role; confirm usage before removing.
- Make card anatomy predictable: name, type/status, primary reading, primary action, optional advanced details. Keep the UUID exact but secondary.
- Let simple cards stay compact. Large stepper/RFID/LiDAR work should have deliberate layout space rather than stretching neighboring cards into empty columns.
- Show short recovery guidance with technical details available on demand. Keep unknown, draft, command sent, and physically reported states visibly distinct.
- Preserve keyboard focus, tooltip meaning, motion preferences and adequate control spacing; avoid using low opacity as the only state indication.

## Proposed dashboard hierarchy

```text
Persistent shell: navigation | current route | device connection/auth | theme
Dashboard
  Disconnected: useful connection setup + short explanation of next step
  Connected: device summary + compact connection controls + expandable details
  Modules
    readable name / type / state
    primary reported metric + clear command/target controls
    optional advanced settings and exact identifier
  Secondary system/security details (warnings stay visible when relevant)
```

Warnings and errors should not be buried inside a closed disclosure. First-use guidance should help someone connect a device, not display fabricated metrics.

## Reusable UI recipes

| Pattern | Purpose |
|---|---|
| PageHeader / SectionHeader | Consistent title, short context and relevant actions; current header repeats a generic label |
| ConnectionSummary / StatusBadge | Shared semantic presentation of connection/auth, including Listening vs Connected |
| EmptyState / LoadingState / ErrorState | Useful reason and next action; keep heading and context stable |
| ModuleCard / MetricRow / AdvancedSettings | Consistent identity, readable readings and progressive disclosure |
| CommandFeedback / FieldHelp | Pending/result/failure and valid draft guidance, accessible without console access |

Implement only the recipes that have multiple real consumers; these names are planning concepts, not a mandate for a large abstraction layer.

## Design review criteria

At 800×600, a user should quickly identify whether the device is connected, how to connect/recover, and where module controls live. At 1440×900, information should use the available width without becoming a wall of equally weighted cards. At 390px and 200% zoom, content should reflow with controls and focus reachable.

Review light/dark examples for disconnected, connected-with-modules, read-only, command failure, empty logs, populated logs, no LiDAR, partial LiDAR, and full LiDAR. Existing screenshot evidence is the baseline; new before/after evidence belongs under a new run directory. Aesthetic choices such as final accent hue and card depth can be settled in UI Phase 1 without blocking this completed audit.
