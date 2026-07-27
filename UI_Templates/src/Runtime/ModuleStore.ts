import { create } from "zustand";
import { Commandtype } from "src/shared/Protocol/ModuleCommand";
import { ModuleEventEnvelope } from "src/shared/Protocol/ModuleEven";
import {
  ModuleDefinitionType,
  Registration,
  SystemInfoType,
} from "src/shared/Protocol/ModuleDefinitionSchema";
import { buttonInitialBuild, updateButton } from "../mainview/Modules/button/definition";
import { ledInitialBuild, updateLed } from "../mainview/Modules/led/definition";
import { lidarInitialBuild, updateLidar } from "../mainview/Modules/Lidar/definition";
import {
  rangefinderInitialBuild,
  updateRangefinder,
} from "../mainview/Modules/rangefinder/definition";
import { servoInitialBuild, updateServo } from "../mainview/Modules/servo/definition";
import { electroview } from "@/electrobun";


export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error" | '';


type ModuleStore = {
  modules: Record<string, ModuleDefinitionType>;
  LookUp_ID_refTo_ID: Record<string, string>;
  portInfo: {
    path: string,
    status: ConnectionStatus
  };
  SystemInfo: SystemInfoType | null;
  setSystemInfo: (sys: SystemInfoType) => void;
  setPort: (path: string, status: ConnectionStatus) => void

  registerModule: (registration: Registration) => void;
  dispatchModuleEvent: (event: ModuleEventEnvelope) => void;
  sendCommand: (command: Commandtype) => Promise<void>;

  ModuleCount: () => number,
  reset: () => void
};


export const useModuleStore = create<ModuleStore>((set, get) => ({
  modules: {},
  SystemInfo: null,
  LookUp_ID_refTo_ID: {},
  portInfo: {
    path: "",
    status: "disconnected",

  },

  registerModule: (registration) => {

    const module_has = get().modules[registration.id]
    if (module_has) {
      return
    }
    console.debug(`id :${registration.id} | module_type :${registration.module_type} | lool_up_id :${registration.lool_up_id}  | parent_id :${registration.parent_id}`)


    const module = createModule(registration);
    if (!module) {
      return;
    }
    set((store) => ({
      modules: {
        ...store.modules,
        [registration.id]: module,
      },
      LookUp_ID_refTo_ID: {
        ...store.LookUp_ID_refTo_ID,
        [registration.lool_up_id]: registration.id
      }
    }));
  },

  dispatchModuleEvent: (event) => {
    set((store) => {
      if (event.module_type === "SysLog") {
        return store;
      }

      const id = event.event.id;
      const module = store.modules[id];

      if (!module) {
        return store;
      }

      const nextModule = applyModuleEvent(module, event);

      if (module === nextModule) {
        return store;
      }

      return {
        modules: {
          ...store.modules,
          [id]: nextModule,
        },
      };
    });
  },

  sendCommand: async (command) => {
    await electroview.rpc?.request.sendComand(command)
  },


  ModuleCount: () => {
    console.log(Object.values(get().modules))
    return Object.values(get().modules).length
  },
  setPort(path, status) {
    set({
      portInfo: {
        path,
        status
      }
    })

  },
  setSystemInfo(sys) {
    set({
      SystemInfo: sys,
    });
  },

  reset() {
    set({
      modules: {},
      LookUp_ID_refTo_ID: {}

    })

  },
}));

function applyModuleEvent(
  module: ModuleDefinitionType,
  event: Exclude<ModuleEventEnvelope, { module_type: "SysLog" }>,
): ModuleDefinitionType {
  switch (event.module_type) {
    case "Led":
      return updateLed(module, event.event);

    case "Button":
      return updateButton(module, event.event);

    case "Servo":
      return updateServo(module, event.event);

    case "Lidar":
      return updateLidar(module, event.event);

    case "Rangefinder":
      return updateRangefinder(module, event.event);
  }
}

function createModule(
  registration: Registration,
): ModuleDefinitionType | undefined {
  switch (registration.module_type) {
    case "Led":
      return ledInitialBuild(
        registration.id,
        registration.parent_id,
        registration.lool_up_id,
      );

    case "Button":
      return buttonInitialBuild(
        registration.id,
        registration.parent_id,
        registration.lool_up_id,
      );

    case "Servo":
      return servoInitialBuild(
        registration.id,
        registration.parent_id,
        registration.lool_up_id,
      );

    case "Lidar":
      return lidarInitialBuild(
        registration.id,
        registration.parent_id,
        registration.lool_up_id,
      );

    case "Rangefinder":
      return rangefinderInitialBuild(
        registration.id,
        registration.parent_id,
        registration.lool_up_id,
      );

    default:
      return undefined;
  }
}
