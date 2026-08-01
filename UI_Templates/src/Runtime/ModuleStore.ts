import { create } from "zustand";
import { Commandtype } from "@shared/Protocol/ModuleCommand";
import { ModuleEventEnvelope } from "@shared/Protocol/ModuleEven";
import {
  ModuleDefinitionType,
  Registration,
  SystemInfoType,
  TypeIdentifier,
  TypeIdentifier_module,
} from "@shared/Protocol/ModuleDefinitionSchema";
import { buttonInitialBuild, updateButton } from "@modules/button/definition";
import { ledInitialBuild, updateLed } from "@modules/led/definition";
import { lidarInitialBuild, updateLidar } from "@modules/Lidar/definition";
import {
  rangefinderInitialBuild,
  updateRangefinder,
} from "@modules/rangefinder/definition";
import { servoInitialBuild, updateServo } from "@modules/servo/definition";
import { electroview } from "@/electrobun";
import { updateStepperMotor , stepperMotorInitialBuild } from "@modules/stepper/definition";

type ModuleByType<T extends TypeIdentifier_module> = Extract<
  ModuleDefinitionType,
  { module_type: T }
>;

type MutableStatePatchByType<T extends TypeIdentifier_module> =
  ModuleByType<T> extends infer TModule
    ? TModule extends ModuleDefinitionType
      ? TModule["mutableStateFields"] extends readonly (infer TKey)[]
        ? Partial<
            Pick<
              TModule["state"],
              Extract<TKey, keyof TModule["state"]>
            >
          >
        : never
      : never
    : never;

type CreateModuleStateUpdater = <T extends TypeIdentifier_module>(
  moduleType: T,
  id: string,
) => (patch: MutableStatePatchByType<T>) => void;

export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error" | '';

type ModuleDefinitionTypeSim = {
  id: string,
  lookUpId: string,
  module_type: TypeIdentifier,
  parent_id: string,
  has_parent: boolean

}

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

  ModuleFilterCategory: () => {
    StandAlone: ModuleDefinitionTypeSim[],
    Grouping: ModuleDefinitionTypeSim[],
  },
  createModuleStateUpdater: CreateModuleStateUpdater;
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

  ModuleFilterCategory() {
    const List = Object.values(get().modules)
    return {
      StandAlone: List.filter(item => item.parent_id.length > 10)
        .map(item => ({

          id: item.id,
          lookUpId: item.lool_up_id,
          module_type: item.module_type,
          has_parent: item.parent_id.length > 10,
          parent_id: item.parent_id,


        }))
      ,

      Grouping: List.filter(item => item.parent_id.length >= 0)
        .map(item => ({

          id: item.id,
          lookUpId: item.lool_up_id,
          module_type: item.module_type,
          has_parent: item.parent_id.length > 10,
          parent_id: item.parent_id,


        }))
    }

  },
  createModuleStateUpdater: ((
    moduleType: TypeIdentifier_module,
    id: string,
  ) => {
    return (patch: Record<string, unknown>) => {
      set((state) => {
        const module = state.modules[id];

        if (!module) {
          return {};
        }

        if (module.module_type !== moduleType) {
          console.error(
            `Module "${id}" is "${module.module_type}", not "${moduleType}".`,
          );
          return {};
        }

        const mutableFields = new Set(
          module.mutableStateFields as readonly string[],
        );

        const invalidField = Object.keys(patch).find(
          (field) => !mutableFields.has(field),
        );

        if (invalidField) {
          console.error(
            `State field "${invalidField}" is not mutable on module "${moduleType}".`,
          );
          return {};
        }

        const updatedModule = {
          ...module,
          state: {
            ...module.state,
            ...patch,
          },
        } as ModuleDefinitionType;

        return {
          modules: {
            ...state.modules,
            [id]: updatedModule,
          },
        };
      });
    };
  }) as CreateModuleStateUpdater,
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
    case "StepperMotor":
      return updateStepperMotor(module, event.event);
    default:
      return module;
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

    case "StepperMotor":
      return stepperMotorInitialBuild(
        registration.id,
        registration.parent_id,
        registration.lool_up_id,
      );

    default:
      return undefined;
  }
}
