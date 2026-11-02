// Types shared by the store slices (audit Phase 2D, PIN-TPL-020).
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { StateCreator, StoreApi } from "zustand";
import type { StoreApi as VanillaStoreApi } from "zustand/vanilla";

import type { DeviceAuth } from "../auth";
import type { ButtonModule } from "../Modules/button";
import type { ImuModule } from "../Modules/imu";
import type { LedModule } from "../Modules/Led";
import type { LidarModule } from "../Modules/lidar";
import type { RangefinderModule } from "../Modules/rangefinder";
import type { RemoteReceiverModule } from "../Modules/remote-receiver";
import type { RfidModule } from "../Modules/rfid";
import type { ServoModule } from "../Modules/servo";
import type { StepperModule } from "../Modules/stepper";
import type { EventPackage } from "../protocol/event";
import type { ProtocolMessage } from "../protocol/message";
import type { Registration } from "../protocol/registration";
import type { SystemInfo } from "../protocol/system";

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

export type PortConnectionType = "Connected" | "Disconnected" | "Listening" | "Error";
export type TransportMode = "Serial" | "Wi-Fi" | "Bluetooth";
export type WifiListenerStatus = { listening: boolean; connected: boolean; port: number };
export type ModuleRegistry = { [id: string]: VanillaStoreApi<AnyModule> };

/** Connection lifecycle and Tauri listeners (the transport service). */
export interface TransportSlice {
  PortStat: PortConnectionType;
  TransportMode: TransportMode | null;
  PortError: string | null;
  PortState: { name: string; rate: number };
  WifiPort: number | null;
  UnlistenFnArr: UnlistenFn[];
  Connect: (portName: string, rate: number) => Promise<void>;
  ConnectBluetooth: (portName: string) => Promise<void>;
  ConnectWifi: (port: number) => Promise<void>;
  RefreshWifiStatus: () => Promise<void>;
  Disconnect: () => Promise<void>;
  InitializeListeners: () => Promise<void>;
}

/** What the device told us: modules, lookup names, system snapshot. */
export interface RegistrySlice {
  ModuleRegistry: ModuleRegistry;
  LookUpId: Record<string, string>;
  /** Lookup names registered by more than one module (PIN-TPL-017). */
  DuplicateLookUpIds: string[];
  systemInfo: SystemInfo | null;
  HandleEvent: (event: ProtocolMessage) => void;
  RegistrationEvent: (event: Registration) => void;
  ModuleEvent: (event: EventPackage) => void;
  SystemEvent: (event: SystemInfo) => void;
  BuildViewCache: () => { id: string; look_up_id: string; kind: string }[];
}

/** The bounded, batched protocol log history (PIN-TPL-011). */
export interface LogSlice {
  logs: ProtocolMessage[];
  clearLogs: () => void;
  /** Queues one message for the next batched append. */
  pushLog: (message: ProtocolMessage) => void;
}

/** Device authentication status mirrored from the backend (Phase 2B). */
export interface AuthSlice {
  DeviceAuth: DeviceAuth | null;
  PairingError: string | null;
  PairDevice: () => Promise<void>;
  UnpairDevice: () => Promise<void>;
}

export type StoreState = TransportSlice & RegistrySlice & LogSlice & AuthSlice;
export type Slice<T> = StateCreator<StoreState, [], [], T>;
export type Store = StoreApi<StoreState>;

/** Everything learned from the device; reset whenever the transport changes. */
export function emptyDeviceState(): Pick<
  StoreState,
  "systemInfo" | "ModuleRegistry" | "LookUpId" | "DuplicateLookUpIds" | "DeviceAuth" | "PairingError"
> {
  return {
    systemInfo: null,
    ModuleRegistry: {},
    LookUpId: {},
    DuplicateLookUpIds: [],
    DeviceAuth: null,
    PairingError: null,
  };
}
