# Critical findings

Part of the [issue register](README.md).

**No Critical findings were confirmed in the current firmware image at commit `5b5c8ed4`.**

The active image drives only three PWM LEDs over Bluetooth SPP. The defects found in it can corrupt LED output, reboot debug builds, stop command processing or crash the desktop log view (PIN-TPL-001, -002, -003, -006). None of these can cause dangerous physical behavior with LEDs alone.

## Escalation watch-list

These findings must be re-rated **Critical** when the named condition becomes true, for example when the code is uncommented in `Firmware_Templates/src/main.rs`:

| Finding | Becomes Critical when | Why |
|---|---|---|
| [PIN-TPL-004](high.md) | A servo, stepper or LiDAR module is instantiated | Motion nobody commanded at boot and after every reset; unbounded travel on command |
| [PIN-TPL-003](high.md) | Any actuator is reachable over Bluetooth | Unauthenticated nearby devices can drive moving hardware |
| [PIN-TPL-001](high.md) | An actuator is driven by an external controller that holds its last output (e.g. PCA9685) | Command processing stops while the actuator keeps its last output; no stop command can be delivered **Cleared in Phase 1B:** command failures no longer stop the loop. |
| [PIN-TPL-005](high.md) | LiDAR is instantiated | A non-terminating scan ends in heap exhaustion, a reboot, and then the boot-time servo sweep |

**Reassessment 2026-10-31:** no change. All four entries are still latent (the code is not instantiated), and the roadmap adds a safety gate: none of these modules may be enabled until Phases 1B, 1C and 1D are Completed — Verified.

**Phase 1B 2026-10-31:** PIN-TPL-001 resolved, so its entry no longer applies. PIN-TPL-003, -004 and -005 remain on the list until 1C and 1D.

**Phase 1C 2026-10-31:** PIN-TPL-003 is reduced, not cleared: new pairings are limited to 120 s after boot, but a device paired in that window can still drive any registered actuator. The entry stays until application-level authentication (2B).

**Phase 1D 2026-10-31:** PIN-TPL-004 and PIN-TPL-005 resolved (no motion at construction, bounded stepper targets, terminating LiDAR scans), so their entries no longer apply. Only PIN-TPL-003 remains on this list. The roadmap safety gate (1B, 1C, 1D Completed — Verified) is met for hardware-independent checks; keep actuators out of the default image until the hardware checks pass and, preferably, until 2B closes PIN-TPL-003.

**Phase 2A 2026-11-01:** no entry added or removed. Note for PIN-TPL-003: child routing now lets a connected peer address a composite's children directly (for example the LiDAR servos). The LiDAR is still not instantiated, so this stays latent, and the 2B authentication covers it.

**Phase 2B 2026-11-02:** PIN-TPL-003 resolved for paired devices, so its entry no longer applies to a device that has been paired over USB: a peer that cannot authenticate is read-only, and commands are signed. The list is now empty. Condition for enabling actuators: **pair the device first**. An unpaired device is still open by design, and the UI warns about it.

Any future Critical finding is recorded here with its full template, and its history is kept after resolution.
