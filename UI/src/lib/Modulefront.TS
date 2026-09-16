import {
  createButton,
  type ButtonModule,
  type ButtonEvent,
} from "./Modules/button";

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
//import { createLed, LedEvent, type LedModule } from "./Modules/Led";

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
import { recordLogReceipt } from "./logs";

export function registrationHasParent(registration: Registration): boolean {
  // lidar.rs creates two servos and a rangefinder; joystick.rs owns a button
  // (the joystick implementation is currently excluded from the firmware build).
  // if (
  //   registration.module_type === "Lidar" ||
  //   registration.module_type === "JoyStick"
  // ) {
  //   return false;
  // }
  // ModuleCore::new serializes None as an empty string. Also tolerate older
  // registrations with missing/null parent_id without changing the wire schema.
  return (
    typeof registration.parent_id === "string" &&
    registration.parent_id.length > 0
  );
}

export type AnyModule =
  // | LedModule
  | ButtonModule
  

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
        console.info("Raw message received:", message);
        get().HandleEvent(message);

      } catch (error) {
        console.error("Failed to handle Pinora message:", message, error);
      }
    });
    set({ UnlistenFnArr: [...get().UnlistenFnArr, unlisten] });
  },
  HandleEvent: (event: ProtocolMessage) => {
    recordLogReceipt(event);
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
      const factories = {
        //Led: createLed,
        Button: createButton,
        
      };
      const factory = factories[event.module_type as keyof typeof factories];
      if (!factory) return;
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
      // if (eventKindType === "Led")
      //   (moduleStore as StoreApi<LedModule>).getState().handleEvent(eventData as LedEvent);
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

}));
