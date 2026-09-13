import {
  createButton,
  type ButtonModule,
  type ButtonEvent,
} from "./Modules/button";
import { createImu, type ImuModule, type ImuEvent } from "./Modules/imu";
import {
  createRemoteReceiver,
  type RemoteReceiverModule,
  type RemoteButtonEvent,
} from "./Modules/remote-receiver";
import {
  createRangefinder,
  type RangefinderModule,
  type RangefinderEvent,
} from "./Modules/rangefinder";
import { createRfid, type RfidModule, type RfidEvent } from "./Modules/rfid";
import {
  createStepper,
  type StepperModule,
  type StepperMotorEvent,
} from "./Modules/stepper";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { createLed, LedEvent, type LedModule } from "./Modules/Led";
import { createLidar, LidarEvent, type LidarModule } from "./Modules/lidar";
import { createServo, ServoEvent, type ServoModule } from "./Modules/servo";
import { StoreApi } from "zustand/vanilla";
import { create } from "zustand";
import { ModuleType } from "./protocol/module-type";
import {
  ProtocolMessage,
  ProtocolMessageKey,
  ProtocolMessageValue,
} from "./protocol/message";
import { Registration } from "./protocol/registration";
import { SystemInfo } from "./protocol/system";
import { EventPackage } from "./protocol/event";

export type AnyModule =
  | LedModule
  | LidarModule
  | ServoModule
  | ButtonModule
  | ImuModule
  | RemoteReceiverModule
  | RangefinderModule
  | RfidModule
  | StepperModule;

type PortConnectionType = "Connected" | "Disconnected" | "Error";

type ModuleRegistry = {
  [id: string]: StoreApi<AnyModule>;
};

interface ModuleCoreArchive {
  PortStat: PortConnectionType;
  PortError: string | null;
  PortState: {
    name: string;
    rate: number;
  };
  UnlistenFnArr: UnlistenFn[];

  Connect: (portName: string, rate: number) => Promise<void>;
  Disconnect: () => Promise<void>;
  InitializeListeners: () => Promise<void>;

  HandleEvent: (event: ProtocolMessage) => void;
  RegistrationEvent: (event: Registration) => void;
  ModuleEvent: (event: EventPackage) => void;
  SystemEvent: (event: SystemInfo) => void;
  BuildViewCache: () => { id: string; look_up_id: string; kind: string }[];

  ModuleRegistry: ModuleRegistry;
  LookUpId: Record<string, string>;
  logs: ProtocolMessage[];
  systemInfo: SystemInfo | null;
  clearLogs: () => void;

  MakeFakeLidar: () => void;
}

export const useModuleFront = create<ModuleCoreArchive>((set, get) => ({
  PortStat: "Disconnected",
  PortError: null,
  PortState: {
    name: "",
    rate: 0,
  },
  UnlistenFnArr: [],
  Connect: async (portName: string, rate: number) => {
    // Implementation for connecting to a port
    await get().Disconnect();
    await get().InitializeListeners();

    await invoke("start_port", { name: portName, rate: rate })
      .then((result) => {
        console.log("Connection result:", result);
        set({
          PortError: null,
          PortStat: "Connected",
          PortState: { name: portName, rate: rate },
          systemInfo: null,
          logs: [],
          ModuleRegistry: {},
          LookUpId: {},
          
        });
      })
      .catch((error) => {
        console.error("Connection error:", error);
        set({ PortStat: "Error", PortError: error.toString() });
      });
  },
  Disconnect: async () => {
    set({ systemInfo: null });
    try {
      await invoke("stop_port");

      console.log("Disconnected from port");

      set({
        PortStat: "Disconnected",
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

      try {
        get().HandleEvent(message);
      } catch (error) {
        console.error("Failed to handle Pinora message:", message, error);
      }
    });
    set({ UnlistenFnArr: [...get().UnlistenFnArr, unlisten] });
  },
  HandleEvent: (event: ProtocolMessage) => {
    set((state) => ({ logs: [...state.logs, event] }));
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
      const value = event;
      switch (value.module_type) {
        case "Led":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createLed({
                id: value.id as string,
                kind: "Led",
                state: 0,
                look_up_id: value.lool_up_id as string,
              }) as StoreApi<AnyModule>,
            },
            LookUpId: {
              ...get().LookUpId,
              [value.lool_up_id]: value.id as string,
            },
          });

          break;

        case "Lidar":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createLidar({
                id: value.id as string,
                kind: "Lidar",
                look_up_id: value.lool_up_id as string,
                state: {
                  Roi: { min: { x: 0, y: 0 }, max: { x: 0, y: 0 } },
                  PointMap: { max_chunk: 0, curr_chunk: 0, map: [] },
                  Target: { point: { x: 0, y: 0 } },
                  ScanState: { state: "StopScan", scan_time: 0 },
                },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: {
              ...get().LookUpId,
              [value.lool_up_id]: value.id as string,
            },
          });

          break;

        case "Servo":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createServo({
                id: value.id as string,
                kind: "Servo",
                look_up_id: value.lool_up_id as string,
                state: { MinPivot: 0, MaxPivot: 0, Offset: 0, Angle: 0 },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: {
              ...get().LookUpId,
              [value.lool_up_id]: value.id as string,
            },
          });

          break;
        case "Button":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createButton({
                id: value.id,
                kind: "Button",
                look_up_id: value.lool_up_id,
                state: { clickCount: 0 },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: { ...get().LookUpId, [value.lool_up_id]: value.id },
          });
          break;
        case "Imu":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createImu({
                id: value.id,
                kind: "Imu",
                look_up_id: value.lool_up_id,
                state: { Gyro: null, Accel: null, Mode: null },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: { ...get().LookUpId, [value.lool_up_id]: value.id },
          });
          break;
        case "RemoteReceiver":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createRemoteReceiver({
                id: value.id,
                kind: "RemoteReceiver",
                look_up_id: value.lool_up_id,
                state: { key: null, clickCount: 0 },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: { ...get().LookUpId, [value.lool_up_id]: value.id },
          });
          break;
        case "Rangefinder":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createRangefinder({
                id: value.id,
                kind: "Rangefinder",
                look_up_id: value.lool_up_id,
                state: {
                  Range: null,
                  RangingState: null,
                  TimingBudget: null,
                  DistanceMode: null,
                  InvalidMeasurement: null,
                },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: { ...get().LookUpId, [value.lool_up_id]: value.id },
          });
          break;
        case "Rfid":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createRfid({
                id: value.id,
                kind: "Rfid",
                look_up_id: value.lool_up_id,
                state: { GetCard: null, GetMode: null, GetWriteState: null },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: { ...get().LookUpId, [value.lool_up_id]: value.id },
          });
          break;
        case "StepperMotor":
          set({
            ModuleRegistry: {
              ...get().ModuleRegistry,
              [value.id]: createStepper({
                id: value.id,
                kind: "StepperMotor",
                look_up_id: value.lool_up_id,
                state: {
                  GetAngle: null,
                  GetPivotMin: null,
                  GetPivotMax: null,
                  GetMode: null,
                  GetOrigin: null,
                  GetPivotPoint: null,
                },
              }) as StoreApi<AnyModule>,
            },
            LookUpId: { ...get().LookUpId, [value.lool_up_id]: value.id },
          });
          break;
        default:
          break;
      }
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
      if (eventKindType === "Button")
        (moduleStore as StoreApi<ButtonModule>)
          .getState()
          .handleEvent(eventData as ButtonEvent);
      if (eventKindType === "Imu")
        (moduleStore as StoreApi<ImuModule>)
          .getState()
          .handleEvent(eventData as ImuEvent);
      if (eventKindType === "RemoteReceiver")
        (moduleStore as StoreApi<RemoteReceiverModule>)
          .getState()
          .handleEvent(eventData as RemoteButtonEvent);
      if (eventKindType === "Rangefinder")
        (moduleStore as StoreApi<RangefinderModule>)
          .getState()
          .handleEvent(eventData as RangefinderEvent);
      if (eventKindType === "Rfid")
        (moduleStore as StoreApi<RfidModule>)
          .getState()
          .handleEvent(eventData as RfidEvent);
      if (eventKindType === "StepperMotor")
        (moduleStore as StoreApi<StepperModule>)
          .getState()
          .handleEvent(eventData as StepperMotorEvent);
      if (eventKindType === "Led") {
        const ledStore = moduleStore as StoreApi<LedModule>;
        if (ledStore) {
          const ledState = ledStore.getState();
          ledState.handleEvent(eventData as LedEvent);
          // Handle LED-specific event here
        }
      }
      if (eventKindType === "Lidar") {
        const lidarStore = moduleStore as StoreApi<LidarModule>;
        if (lidarStore) {
          const lidarState = lidarStore.getState();
          lidarState.handleEvent(eventData as LidarEvent);
          // Handle Lidar-specific event here
        }
      }
      if (eventKindType === "Servo") {
        const servoStore = moduleStore as StoreApi<ServoModule>;
        if (servoStore) {
          const servoState = servoStore.getState();
          servoState.handleEvent(eventData as ServoEvent);
          // Handle Servo-specific event here
        }
      }

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
  logs: [],
  systemInfo: null,
  clearLogs: () => set({ logs: [] }),
  BuildViewCache: (): { id: string; look_up_id: string; kind: string }[] => {
    // Implement the logic to build the view cache here
    return Object.entries(get().LookUpId).map(([look_up_id, id]) => ({
      id,
      look_up_id,
      kind: get().ModuleRegistry[id]?.getState().kind ?? "",
    }));
  },
  MakeFakeLidar: () => {
    set({
      ModuleRegistry: {
        ...get().ModuleRegistry,
        "fake-lidar": createLidar({
          id: "fake-lidar",
          kind: "Lidar",
          look_up_id: "fake-lidar",
          state: {
            Roi: { min: { x: 0, y: 0 }, max: { x: 0, y: 0 } },
            PointMap: { max_chunk: 0, curr_chunk: 0, map: [] },
            Target: { point: { x: 0, y: 0 } },
            ScanState: { state: "StopScan", scan_time: 0 },
          },
        }) as StoreApi<AnyModule>,
      },
      LookUpId: {
        ...get().LookUpId,
        "fake-lidar": "fake-lidar",
      },
    });
    console.log("Fake Lidar created", get().ModuleRegistry["fake-lidar"]);
  },
}));
