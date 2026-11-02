// Device authentication status shown by the UI (audit Phase 2B, PIN-TPL-003).
// Payload shapes match the Rust AuthStatus / PairingEvent serialization
// (`#[serde(tag = "state")]`, checked in protocol/tests/auth.rs).
//
// Run from UI/: `bun test`.
import { describe, expect, test } from "bun:test";
import { AUTH_PROTOCOL_VERSION, DeviceAuthSchema, describeAuth } from "../src/lib/auth";

const device = "a1b2c3d4e5f6";

describe("deviceAuth payloads", () => {
  test("every backend state parses", () => {
    for (const payload of [
      { state: "Idle" },
      { state: "Pending" },
      { state: "Authenticated", device_id: device },
      { state: "Unprovisioned", device_id: device },
      { state: "NoKey", device_id: device },
      { state: "DeviceRejected", device_id: device },
      { state: "Failed", reason: "authentication failed" },
      { state: "Paired", device_id: device },
      { state: "KeyCleared", device_id: device },
      { state: "PairingFailed", reason: "no USB" },
    ]) {
      expect(DeviceAuthSchema.safeParse(payload).success).toBe(true);
    }
  });

  test("a payload carrying a key, or an unknown state, is rejected", () => {
    expect(DeviceAuthSchema.safeParse({ state: "Paired", device_id: device, key: "11" }).success).toBe(false);
    expect(DeviceAuthSchema.safeParse({ state: "Owned" }).success).toBe(false);
    expect(DeviceAuthSchema.safeParse(null).success).toBe(false);
  });
});

describe("describeAuth", () => {
  test("authenticated links are ok; unpaired or read-only links warn; failures are errors", () => {
    const tone = (state: object) => describeAuth(DeviceAuthSchema.parse(state), "Bluetooth", 2).tone;
    expect(tone({ state: "Authenticated", device_id: device })).toBe("ok");
    expect(tone({ state: "Unprovisioned", device_id: device })).toBe("warning");
    expect(tone({ state: "NoKey", device_id: device })).toBe("warning");
    expect(tone({ state: "DeviceRejected", device_id: device })).toBe("error");
    expect(tone({ state: "Failed", reason: "x" })).toBe("error");
    expect(tone({ state: "PairingFailed", reason: "x" })).toBe("error");
  });

  test("details name the device and tell the user how to pair", () => {
    const summary = describeAuth({ state: "NoKey", device_id: device }, "Wi-Fi", 2);
    expect(summary.detail).toContain(device);
    expect(summary.detail).toContain("USB");
  });

  test("pre-v2 firmware over a radio link is flagged; USB is not", () => {
    expect(describeAuth(null, "Bluetooth", 1).title).toBe("Authentication not supported");
    expect(describeAuth(null, "Bluetooth", 1).detail).toContain(`v${AUTH_PROTOCOL_VERSION}`);
    expect(describeAuth(null, "Serial", 1).title).toBe("USB serial");
  });

  test("never throws without a status", () => {
    expect(describeAuth(null, null, null).tone).toBe("info");
  });
});
