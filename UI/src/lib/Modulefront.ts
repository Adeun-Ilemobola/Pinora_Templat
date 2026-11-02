// useModuleFront: the desktop's central store.
//
// It owns the transport connection lifecycle (Serial, Bluetooth SPP, Wi-Fi), the
// Tauri event listeners, the module registry (UUID -> per-module vanilla store),
// the lookup-name map, the protocol log history and the latest system snapshot.
//
// Incoming espState lines have been validated as Rust ProtocolMessages by the Tauri
// backend (all transports since Phase 1C); here they are validated again with the
// Zod ProtocolMessageSchema (Phase 2A, PIN-TPL-007) and routed by their top-level
// key. Non-protocol lines arrive separately on espConsole. Only Led and Button
// registrations create stores. The log history is bounded and batched (Phase 2D,
// PIN-TPL-011: before, it was unbounded and copied on every message). IPC
// command and event names are contracts with the Tauri backend (C-06, C-07).
//
// Rediscovery (Phase 2A, PIN-TPL-012): state is cleared before a transport starts,
// and a Rediscover system command is sent once it is connected, so the device
// re-sends its snapshot, registrations and current state on every transport.
//
// Authentication (Phase 2B, PIN-TPL-003): the Tauri backend runs the handshake and
// keeps the keys; this store only mirrors the `deviceAuth` status and asks the
// backend to pair or unpair over USB serial.
import {
  createButton,
  type ButtonModule,
  type ButtonEvent,
} from "./Modules/button";

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { createLed, LedEvent, type LedModule } from "./Modules/Led";
import type { ImuModule } from "./Modules/imu";
import type { LidarModule } from "./Modules/lidar";
import type { RangefinderModule } from "./Modules/rangefinder";
import type { RemoteReceiverModule } from "./Modules/remote-receiver";
import type { RfidModule } from "./Modules/rfid";
import type { ServoModule } from "./Modules/servo";
import type { StepperModule } from "./Modules/stepper";

import { StoreApi } from "zustand/vanilla";
import { create } from "zustand";
import { ModuleType } from "./protocol/module-type";
import {
  ProtocolMessage,
  ProtocolMessageKey,
  ProtocolMessageSchema,
  ProtocolMessageValue,
} from "./protocol/message";
import { Registration } from "./protocol/registration";
import { SystemInfo } from "./protocol/system";
import { EventPackage } from "./protocol/event";
import { appendBounded, createLogBatcher, recordLogReceipt } from "./logs";
import { requestRediscovery } from "./IncomingCommand";
import { DeviceAuthSchema, type DeviceAuth } from "./auth";

export { registrationHasParent } from "./registry/registration";
import { registrationHasParent } from "./registry/registration";

export type AnyModule =
  | LedModule
  | ButtonModule
  | ImuModule
  | LidarModule
  | RangefinderModule
  | RemoteReceiverModule
  | RfidModule
  | ServoModule
  | StepperModule;


type PortConnectionType = "Connected" | "Disconnected" | "Listening" | "Error";
type TransportMode = "Serial" | "Wi-Fi" | "Bluetooth";
type WifiListenerStatus = {
  listening: boolean;
  connected: boolean;
  port: number;
};

type ModuleRegistry = {
  [id: string]: StoreApi<AnyModule>;
};

interface ModuleCoreArchive {
  PortStat: PortConnectionType;
  TransportMode: TransportMode | null;
  PortError: string | null;
  PortState: {
    name: string;
    rate: number;
  };
  WifiPort: number | null;
  UnlistenFnArr: UnlistenFn[];

  Connect: (portName: string, rate: number) => Promise<void>;
  ConnectBluetooth: (portName: string) => Promise<void>;
  ConnectWifi: (port: number) => Promise<void>;
  RefreshWifiStatus: () => Promise<void>;
  Disconnect: () => Promise<void>;
  InitializeListeners: () => Promise<void>;

  HandleEvent: (event: ProtocolMessage) => void;
  RegistrationEvent: (event: Registration) => void;
  ModuleEvent: (event: EventPackage) => void;
  SystemEvent: (event: SystemInfo) => void;
  BuildViewCache: () => { id: string; look_up_id: string; kind: string }[];

  ModuleRegistry: ModuleRegistry;
  LookUpId: Record<string, string>;
  /** Lookup names registered by more than one module (PIN-TPL-017). */
  DuplicateLookUpIds: string[];
  /** Latest `deviceAuth` status from the backend (Phase 2B). */
  DeviceAuth: DeviceAuth | null;
  PairingError: string | null;
  PairDevice: () => Promise<void>;
  UnpairDevice: () => Promise<void>;
  logs: ProtocolMessage[];
  systemInfo: SystemInfo | null;
  clearLogs: () => void;

}

/** Everything learned from the device; reset whenever the transport changes. */
function emptyDeviceState() {
  return {
    systemInfo: null,
    ModuleRegistry: {},
    LookUpId: {},
    DuplicateLookUpIds: [] as string[],
    DeviceAuth: null as DeviceAuth | null,
    PairingError: null as string | null,
  };
}

/** Sends Rediscover; a failure is logged, the boot-time registrations still apply. */
function rediscover() {
  requestRediscovery().catch((error) =>
    console.warn("Rediscovery request failed:", error),
  );
}

export const useModuleFront = create<ModuleCoreArchive>((set, get) => ({
  PortStat: "Disconnected",
  TransportMode: null,
  PortError: null,
  PortState: {
    name: "",
    rate: 0,
  },
  WifiPort: null,
  UnlistenFnArr: [],
  Connect: async (portName: string, rate: number) => {
    // Implementation for connecting to a port
    await get().Disconnect();
    // Cleared before the port opens: clearing after it (before Phase 2A) could drop
    // registrations that arrived in between.
    logBatcher.clear();
    set({ TransportMode: "Serial", ...emptyDeviceState(), logs: [] });
    await get().InitializeListeners();

    await invoke("start_port", { name: portName, rate: rate })
      .then((result) => {
        console.log("Connection result:", result);
        set({
          PortError: null,
          PortStat: "Connected",
          PortState: { name: portName, rate: rate },
        });
        rediscover();
      })
      .catch((error) => {
        console.error("Connection error:", error);
        set({ PortStat: "Error", PortError: error.toString() });
      });
  },
  ConnectWifi: async (port: number) => {
    await get().Disconnect();
    logBatcher.clear();
    set({
      TransportMode: "Wi-Fi",
      PortStat: "Listening",
      PortError: null,
      ...emptyDeviceState(),
      logs: [],
    });
    await get().InitializeListeners();
    try {
      await invoke("start_wifi_listener", { port });
      set({ WifiPort: port });
    } catch (error) {
      get().UnlistenFnArr.forEach((unlisten) => unlisten());
      set({ UnlistenFnArr: [] });
      set({ PortStat: "Error", PortError: String(error), TransportMode: null });
      throw error;
    }
  },
  ConnectBluetooth: async (portName: string) => {
    await get().Disconnect();
    logBatcher.clear();
    set({
      TransportMode: "Bluetooth",
      PortStat: "Disconnected",
      PortError: null,
      ...emptyDeviceState(),
      logs: [],
    });
    await get().InitializeListeners();
    try {
      await invoke("start_bluetooth_port", { name: portName });
      set({
        PortStat: "Connected",
        PortState: { name: portName, rate: 115200 },
      });
      rediscover();
    } catch (error) {
      get().UnlistenFnArr.forEach((unlisten) => unlisten());
      set({
        UnlistenFnArr: [],
        PortStat: "Error",
        PortError: String(error),
        TransportMode: null,
      });
      throw error;
    }
  },
  RefreshWifiStatus: async () => {
    let status = await invoke<WifiListenerStatus>("get_wifi_listener_status");
    if (status.listening && get().UnlistenFnArr.length === 0) {
      await get().InitializeListeners();
      status = await invoke<WifiListenerStatus>("get_wifi_listener_status");
    }
    set({
      WifiPort: status.port,
    });
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

      set({
        PortStat: "Disconnected",
        TransportMode: null,
        PortError: null,
        PortState: {
          name: "",
          rate: 0,
        },
      });
    } catch (error) {
      console.error("Disconnection error:", error);

      set({
        PortStat: "Error",
        PortError: String(error),
      });
    }

    get().UnlistenFnArr.forEach((unlisten) => {
      unlisten();
    });

    set({ UnlistenFnArr: [] });
  },
  InitializeListeners: async () => {
    // Implementation for initializing listeners
    let unlisten = await listen<string>("espState", (event) => {
      const payload = event.payload.trim();

      if (!payload) {
        return;
      }

      // Pinora protocol messages are JSON objects.
      if (!payload.startsWith("{")) {
        console.debug("ESP:", payload);
        return;
      }

      let message;

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
    // Non-protocol COM lines (firmware console output, malformed JSON) arrive here
    // since Phase 1C instead of on espState.
    const unlistenConsole = await listen<string>("espConsole", (event) => {
      const line = event.payload.trim();
      if (line) {
        console.debug("ESP:", line);
      }
    });
    const unlistenWifi = await listen<string>("wifiConnection", (event) => {
      if (get().TransportMode === "Wi-Fi") {
        if (event.payload === "Connected") {
          set({ PortStat: "Connected" });
          rediscover();
        } else {
          set({ PortStat: "Listening", ...emptyDeviceState() });
        }
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
        unlisten,
        unlistenConsole,
        unlistenWifi,
        unlistenBluetooth,
        unlistenSerial,
      ],
    });
  },
  HandleEvent: (event: ProtocolMessage) => {
    recordLogReceipt(event);
    logBatcher.push(event);
    // Implementation for handling events
    try {
      const key = Object.keys(event)[0] as ProtocolMessageKey;
      const value = Object.values(event)[0] as ProtocolMessageValue;

      switch (key) {
        case "Registration":
          get().RegistrationEvent(value as Registration);
          break;
        case "ModuleEvent":
          get().ModuleEvent(value as EventPackage);
          break;
        case "System":
          get().SystemEvent(value as SystemInfo);
          break;
      }
    } catch (error) {
      console.error("{HandleEvent} Failed to handle event:", error);
    }
  },

  RegistrationEvent: (event: Registration) => {
    try {
      const factories = {
        Led: createLed,
        Button: createButton,
        
      };
      const factory = factories[event.module_type as keyof typeof factories];
      if (!factory) return;
      const existing = get().ModuleRegistry[event.id];
      if (existing && existing.getState().kind === event.module_type) {
        // Rediscovery re-sends registrations; keep the store (and the mounted view).
        return;
      }
      const owner = get().LookUpId[event.lool_up_id];
      if (owner && owner !== event.id) {
        console.warn("Duplicate module lookup name:", event.lool_up_id, owner, event.id);
        set((state) => ({
          DuplicateLookUpIds: state.DuplicateLookUpIds.includes(event.lool_up_id)
            ? state.DuplicateLookUpIds
            : [...state.DuplicateLookUpIds, event.lool_up_id],
        }));
      }
      const moduleStore = factory(
        registrationHasParent(event),
        event.id,
        event.lool_up_id,
      ) as StoreApi<AnyModule>;
      set((state) => ({
        ModuleRegistry: { ...state.ModuleRegistry, [event.id]: moduleStore },
        LookUpId: { ...state.LookUpId, [event.lool_up_id]: event.id },
      }));
    } catch (error) {
      console.error(
        "{RegistrationEvent} Failed to handle registration event:",
        error,
      );
    }
  },
  ModuleEvent: (data: EventPackage) => {
    // console.log("Received ModuleEvent:", data);
    try {
      const id = data.id;
      const moduleStore = get().ModuleRegistry[id];
      const eventKindType = Object.keys(data.event)[0] as ModuleType;
      const eventData = Object.values(data.event)[0];

      if (!moduleStore) {
        console.error("Module store not found for ID:", id);
        return;
      }
      if (moduleStore.getState().kind !== eventKindType) {
        console.error("Module event kind mismatch:", id, eventKindType);
        return;
      }
      if (eventKindType === "Led")
        (moduleStore as StoreApi<LedModule>).getState().handleEvent(eventData as LedEvent);
      if (eventKindType === "Button")
        (moduleStore as StoreApi<ButtonModule>).getState().handleEvent(eventData as ButtonEvent);
     

      //    console.log("Handling ModuleEvent:", id, eventKindType, eventData);
    } catch (error) {
      console.error("{ModuleEvent} Failed to handle module event:", error);
    }
  },
  SystemEvent: (event: SystemInfo) => {
    set({ systemInfo: event });
  },
  ModuleRegistry: {},
  LookUpId: {},
  DuplicateLookUpIds: [],
  DeviceAuth: null,
  PairingError: null,
  // The result arrives as a deviceAuth event (Paired / KeyCleared / PairingFailed).
  PairDevice: async () => {
    try {
      set({ PairingError: null });
      await invoke("provision_device_key");
    } catch (error) {
      set({ PairingError: String(error) });
    }
  },
  UnpairDevice: async () => {
    try {
      set({ PairingError: null });
      await invoke("clear_device_key");
    } catch (error) {
      set({ PairingError: String(error) });
    }
  },
  logs: [],
  systemInfo: null,
  clearLogs: () => {
    logBatcher.clear();
    set({ logs: [] });
  },
  BuildViewCache: (): { id: string; look_up_id: string; kind: string }[] => {
    // Implement the logic to build the view cache here
    return Object.entries(get().LookUpId).map(([look_up_id, id]) => ({
      id,
      look_up_id,
      kind: get().ModuleRegistry[id]?.getState().kind ?? "",
    }));
  },

}));

// One store update per 50 ms burst of messages, bounded to LOG_CAPACITY entries.
const logBatcher = createLogBatcher<ProtocolMessage>((batch) =>
  useModuleFront.setState((state) => ({ logs: appendBounded(state.logs, batch) })),
);
