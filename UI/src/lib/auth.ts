// Device authentication status as reported by the Tauri backend (protocol v2,
// audit Phase 2B, PIN-TPL-003).
//
// The backend runs the handshake and keeps the keys; the WebView only receives
// `deviceAuth` events with the payloads below (AuthStatus and PairingEvent in Rust),
// and never a key.
import { z } from "zod";

const DeviceId = z.string();

export const DeviceAuthSchema = z.discriminatedUnion("state", [
  z.strictObject({ state: z.literal("Idle") }),
  z.strictObject({ state: z.literal("Pending") }),
  z.strictObject({ state: z.literal("Authenticated"), device_id: DeviceId }),
  z.strictObject({ state: z.literal("Unprovisioned"), device_id: DeviceId }),
  z.strictObject({ state: z.literal("NoKey"), device_id: DeviceId }),
  z.strictObject({ state: z.literal("DeviceRejected"), device_id: DeviceId }),
  z.strictObject({ state: z.literal("Failed"), reason: z.string() }),
  // Pairing over USB serial.
  z.strictObject({ state: z.literal("Paired"), device_id: DeviceId }),
  z.strictObject({ state: z.literal("KeyCleared"), device_id: DeviceId }),
  z.strictObject({ state: z.literal("PairingFailed"), reason: z.string() }),
]);

export type DeviceAuth = z.infer<typeof DeviceAuthSchema>;

export type AuthTone = "ok" | "info" | "warning" | "error";

export type AuthSummary = { tone: AuthTone; title: string; detail: string };

/** First protocol version with authentication. */
export const AUTH_PROTOCOL_VERSION = 2;

/** What to show for the current authentication state. Never throws. */
export function describeAuth(
  auth: DeviceAuth | null,
  transport: "Serial" | "Bluetooth" | "Wi-Fi" | null,
  protocolVersion: number | null,
): AuthSummary {
  if (transport !== "Serial" && protocolVersion !== null && protocolVersion < AUTH_PROTOCOL_VERSION) {
    return {
      tone: "warning",
      title: "Authentication not supported",
      detail: `This firmware speaks protocol v${protocolVersion}; device authentication needs v${AUTH_PROTOCOL_VERSION}. Anyone in range can send it commands.`,
    };
  }
  switch (auth?.state ?? "Idle") {
    case "Idle":
      return transport === "Serial"
        ? {
            tone: "info",
            title: "USB serial",
            detail: "USB is trusted without a handshake. Pair this device here to require authentication over Bluetooth and Wi-Fi.",
          }
        : { tone: "info", title: "Not authenticated", detail: "No authentication has run on this connection." };
    case "Pending":
      return { tone: "info", title: "Authenticating…", detail: "Waiting for the device to answer the handshake." };
    case "Authenticated":
      return { tone: "ok", title: "Authenticated", detail: `Paired device ${deviceId(auth)}. Every command is signed.` };
    case "Unprovisioned":
      return {
        tone: "warning",
        title: "Device not paired",
        detail: `Device ${deviceId(auth)} has no key, so any peer that connects can control it. Connect over USB serial and pair it.`,
      };
    case "NoKey":
      return {
        tone: "warning",
        title: "Read-only",
        detail: `Device ${deviceId(auth)} requires authentication, but this desktop is not paired with it. Connect over USB serial and pair it to send commands.`,
      };
    case "DeviceRejected":
      return {
        tone: "error",
        title: "Device not recognized",
        detail: `Device ${deviceId(auth)} did not prove it holds the paired key. No commands are signed for it. If its key was reset, pair it again over USB.`,
      };
    case "Failed":
      return { tone: "error", title: "Authentication failed", detail: reason(auth) };
    case "Paired":
      return {
        tone: "ok",
        title: "Paired",
        detail: `This desktop is paired with device ${deviceId(auth)}. Reconnect over Bluetooth or Wi-Fi to use it.`,
      };
    case "KeyCleared":
      return {
        tone: "warning",
        title: "Key cleared",
        detail: `Device ${deviceId(auth)} no longer has a key: its commands are not authenticated.`,
      };
    case "PairingFailed":
      return { tone: "error", title: "Pairing failed", detail: reason(auth) };
  }
}

function deviceId(auth: DeviceAuth | null): string {
  return auth && "device_id" in auth ? auth.device_id : "unknown";
}

function reason(auth: DeviceAuth | null): string {
  return auth && "reason" in auth && auth.reason ? auth.reason : "No reason given.";
}
