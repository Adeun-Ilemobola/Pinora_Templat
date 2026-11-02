// The transport service: connection lifecycle and Tauri listeners (audit Phase 2D
// split). IPC command and event names are contracts with the Tauri backend (C-06,
// C-07).
//
// Incoming espState lines have been validated as Rust ProtocolMessages by the backend
// (all transports since Phase 1C); here they are validated again with the Zod
// ProtocolMessageSchema (Phase 2A, PIN-TPL-007). Non-protocol lines arrive on
// espConsole.
//
// Rediscovery (Phase 2A, PIN-TPL-012): device state and logs are cleared before a
// transport starts, and Rediscover is sent once it is connected, so the device
// re-sends its snapshot, registrations and current state on every transport.
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import { DeviceAuthSchema } from "../auth";
import { requestRediscovery } from "../IncomingCommand";
import { ProtocolMessageSchema } from "../protocol/message";
import { emptyDeviceState, type Slice, type TransportSlice, type WifiListenerStatus } from "./types";

/** Sends Rediscover; a failure is logged, the boot-time registrations still apply. */
function rediscover() {
  requestRediscovery().catch((error) => console.warn("Rediscovery request failed:", error));
}

export const createTransportSlice: Slice<TransportSlice> = (set, get) => {
  /** Starts a new session: no old device state, no old logs. */
  const freshSession = () => {
    get().clearLogs();
    set(emptyDeviceState());
  };
  const removeListeners = () => {
    get().UnlistenFnArr.forEach((unlisten) => unlisten());
    set({ UnlistenFnArr: [] });
  };

  return {
    PortStat: "Disconnected",
    TransportMode: null,
    PortError: null,
    PortState: { name: "", rate: 0 },
    WifiPort: null,
    UnlistenFnArr: [],

    Connect: async (portName: string, rate: number) => {
      await get().Disconnect();
      // Cleared before the port opens: clearing after it (before Phase 2A) could drop
      // registrations that arrived in between.
      freshSession();
      set({ TransportMode: "Serial" });
      await get().InitializeListeners();
      try {
        const result = await invoke("start_port", { name: portName, rate });
        console.log("Connection result:", result);
        set({ PortError: null, PortStat: "Connected", PortState: { name: portName, rate } });
        rediscover();
      } catch (error) {
        console.error("Connection error:", error);
        set({ PortStat: "Error", PortError: String(error) });
      }
    },

    ConnectWifi: async (port: number) => {
      await get().Disconnect();
      freshSession();
      set({ TransportMode: "Wi-Fi", PortStat: "Listening", PortError: null });
      await get().InitializeListeners();
      try {
        await invoke("start_wifi_listener", { port });
        set({ WifiPort: port });
      } catch (error) {
        removeListeners();
        set({ PortStat: "Error", PortError: String(error), TransportMode: null });
        throw error;
      }
    },

    ConnectBluetooth: async (portName: string) => {
      await get().Disconnect();
      freshSession();
      set({ TransportMode: "Bluetooth", PortStat: "Disconnected", PortError: null });
      await get().InitializeListeners();
      try {
        await invoke("start_bluetooth_port", { name: portName });
        set({ PortStat: "Connected", PortState: { name: portName, rate: 115200 } });
        rediscover();
      } catch (error) {
        removeListeners();
        set({ PortStat: "Error", PortError: String(error), TransportMode: null });
        throw error;
      }
    },

    RefreshWifiStatus: async () => {
      let status = await invoke<WifiListenerStatus>("get_wifi_listener_status");
      if (status.listening && get().UnlistenFnArr.length === 0) {
        await get().InitializeListeners();
        status = await invoke<WifiListenerStatus>("get_wifi_listener_status");
      }
      set({ WifiPort: status.port });
      if (status.listening) {
        set({
          TransportMode: "Wi-Fi",
          PortStat: status.connected ? "Connected" : "Listening",
          PortError: null,
        });
      }
    },

    Disconnect: async () => {
      set(emptyDeviceState());
      try {
        await invoke("stop_port");
        await invoke("stop_wifi_listener");
        console.log("Disconnected from port");
        set({ PortStat: "Disconnected", TransportMode: null, PortError: null, PortState: { name: "", rate: 0 } });
      } catch (error) {
        console.error("Disconnection error:", error);
        set({ PortStat: "Error", PortError: String(error) });
      }
      removeListeners();
    },

    InitializeListeners: async () => {
      const unlistenState = await listen<string>("espState", (event) => {
        const payload = event.payload.trim();
        if (!payload) return;
        // Pinora protocol messages are JSON objects.
        if (!payload.startsWith("{")) {
          console.debug("ESP:", payload);
          return;
        }
        let message: unknown;
        try {
          message = JSON.parse(payload);
        } catch (error) {
          console.error("Malformed JSON:", payload, error);
          return;
        }
        const parsed = ProtocolMessageSchema.safeParse(message);
        if (!parsed.success) {
          // Tauri accepted it, so the Zod schemas lag the Rust protocol crate.
          console.error("Message does not match the protocol schema:", message, parsed.error.issues);
          return;
        }
        try {
          get().HandleEvent(parsed.data);
        } catch (error) {
          console.error("Failed to handle Pinora message:", message, error);
        }
      });
      // Non-protocol COM lines (firmware console output, malformed JSON), since Phase 1C.
      const unlistenConsole = await listen<string>("espConsole", (event) => {
        const line = event.payload.trim();
        if (line) console.debug("ESP:", line);
      });
      const unlistenWifi = await listen<string>("wifiConnection", (event) => {
        if (get().TransportMode !== "Wi-Fi") return;
        if (event.payload === "Connected") {
          set({ PortStat: "Connected" });
          rediscover();
        } else {
          set({ PortStat: "Listening", ...emptyDeviceState() });
        }
      });
      const unlistenBluetooth = await listen<string>("bluetoothConnection", (event) => {
        if (get().TransportMode === "Bluetooth" && event.payload === "Disconnected") {
          set({
            PortStat: "Error",
            PortError: "Bluetooth SPP connection was lost. Disconnect and reconnect.",
            ...emptyDeviceState(),
          });
        }
      });
      // Added in Phase 2A: before, a USB disconnect was not reported (PIN-TPL-012).
      const unlistenSerial = await listen<string>("serialConnection", (event) => {
        if (get().TransportMode === "Serial" && event.payload === "Disconnected") {
          set({
            PortStat: "Error",
            PortError: "USB serial connection was lost. Disconnect and reconnect.",
            ...emptyDeviceState(),
          });
        }
      });
      const unlistenAuth = await listen<unknown>("deviceAuth", (event) => {
        const parsed = DeviceAuthSchema.safeParse(event.payload);
        if (!parsed.success) {
          console.error("Unexpected deviceAuth payload:", event.payload, parsed.error.issues);
          return;
        }
        set({ DeviceAuth: parsed.data, PairingError: null });
      });
      set({
        UnlistenFnArr: [
          ...get().UnlistenFnArr,
          unlistenAuth,
          unlistenState,
          unlistenConsole,
          unlistenWifi,
          unlistenBluetooth,
          unlistenSerial,
        ],
      });
    },
  };
};
