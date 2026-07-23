import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { Command } from "./ModuleCommand";
import { ModuleEventEnvelope } from "./ModuleEven";
import {
  ModuleDefinitionType,
  Registration,
} from "./ModuleDefinitionSchema";
import { buttonInitialBuild, updateButton } from "./Modules/BUTTON";
import { ledInitialBuild, updateLed } from "./Modules/LED";
import { lidarInitialBuild, updateLidar } from "./Modules/LIDAR";
import {
  rangefinderInitialBuild,
  updateRangefinder,
} from "./Modules/RANGEFINDER";
import { servoInitialBuild, updateServo } from "./Modules/SERVO";


type ModuleStore = {
  modules: Record<string, ModuleDefinitionType>;
  LookUp_ID_refTo_ID: Record<string, string>;

  registerModule: (registration: Registration) => void;
  dispatchModuleEvent: (event: ModuleEventEnvelope) => void;
  sendCommand: (command: Command) => Promise<void>;

  ModuleCount: () => number,
};


export const useModuleStore = create<ModuleStore>((set, get) => ({
  modules: {},
  LookUp_ID_refTo_ID: {},

  registerModule: (registration) => {

    const module_has = get().modules[registration.id]
    if (module_has) {
      return
    }


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
    await invoke("send_serial_command", { data: command });
  },

  ModuleCount: () => {
    return Object.values(get().modules).length
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
