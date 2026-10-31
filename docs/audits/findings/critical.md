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
| [PIN-TPL-001](high.md) | An actuator is driven by an external controller that holds its last output (e.g. PCA9685) | Command processing stops while the actuator keeps its last output; no stop command can be delivered |
| [PIN-TPL-005](high.md) | LiDAR is instantiated | A non-terminating scan ends in heap exhaustion, a reboot, and then the boot-time servo sweep |

**Reassessment 2026-10-31:** no change. All four entries are still latent (the code is not instantiated), and the roadmap adds a safety gate: none of these modules may be enabled until Phases 1B, 1C and 1D are Completed — Verified.

Any future Critical finding is recorded here with its full template, and its history is kept after resolution.
