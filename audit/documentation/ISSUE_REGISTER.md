# UI issue register

Canonical index for this UI audit, 2026-11-05. The existing engineering findings retain their PIN-TPL identities under [docs/audits](../../docs/audits/README.md); they are not renumbered or reopened here.

All 13 issues are Open: 9 Medium, 4 Low, 0 High, 0 Critical. Confidence: 12 Confirmed, 1 Probable. Resolved: 0; Verified repairs: 0.

| ID | Finding | Severity | Status | Confidence | UI phase |
|---|---|---|---|---|---|
| [ISS-0001](../issues/medium/ISS-0001.md) | Light-theme secondary text falls below normal-text contrast | medium | Open | Confirmed | 1 |
| [ISS-0002](../issues/medium/ISS-0002.md) | Default desktop layout places module work below large setup cards | medium | Open | Confirmed | 2 |
| [ISS-0003](../issues/medium/ISS-0003.md) | Read-only authentication status does not disable write controls | medium | Open | Confirmed | 3 |
| [ISS-0004](../issues/medium/ISS-0004.md) | LED and servo commands lack local failure and pending feedback | medium | Open | Confirmed | 3 |
| [ISS-0005](../issues/medium/ISS-0005.md) | Partial LiDAR registration produces a blank route | medium | Open | Confirmed | 4 |
| [ISS-0006](../issues/medium/ISS-0006.md) | Virtual log rows reuse positions as message identity | medium | Open | Probable | 4 |
| [ISS-0007](../issues/low/ISS-0007.md) | Corrupted Unicode and raw wire spellings reach UI copy | low | Open | Confirmed | 1 |
| [ISS-0008](../issues/medium/ISS-0008.md) | Current automated checks do not protect rendered UI behavior | medium | Open | Confirmed | 5 |
| [ISS-0009](../issues/low/ISS-0009.md) | Select popovers force dark styling in light mode | low | Open | Confirmed | 1 |
| [ISS-0010](../issues/low/ISS-0010.md) | System theme support does not track preference changes | low | Open | Confirmed | 1 |
| [ISS-0011](../issues/medium/ISS-0011.md) | Module cards omit available human-readable lookup names | medium | Open | Confirmed | 2 |
| [ISS-0012](../issues/medium/ISS-0012.md) | LED slider label is attached to its wrapper rather than its input | medium | Open | Confirmed | 1 |
| [ISS-0013](../issues/low/ISS-0013.md) | Sidebar collapse preference is written but never restored | low | Open | Confirmed | 2 |

Severity ranks impact, not aesthetic preference. New visual features are requirements rather than inflated defect counts. No authentication bypass, hardware failure, cloud defect, or whole-app accessibility conformance result is claimed.
