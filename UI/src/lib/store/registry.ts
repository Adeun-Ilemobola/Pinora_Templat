// The module registry: what the device has told us (audit Phase 2D split).
//
// Protocol messages are routed by their top-level key. Every module family with a
// desktop store gets one when it registers; a family's events go to stores of that
// kind. Re-sent registrations (rediscovery) keep the existing store; duplicate lookup
// names are recorded (PIN-TPL-017).
import type { StoreApi } from "zustand/vanilla";

import { createButton } from "../Modules/button";
import { createImu } from "../Modules/imu/store";
import { createLed } from "../Modules/Led";
import { createLidar } from "../Modules/lidar/store";
import { createRangefinder } from "../Modules/rangefinder/store";
import { createRemoteReceiver } from "../Modules/remote-receiver/store";
import { createRfid } from "../Modules/rfid/store";
import { createServo } from "../Modules/servo/store";
import { createStepper } from "../Modules/stepper/store";
import type { EventPackage } from "../protocol/event";
import type { ProtocolMessage } from "../protocol/message";
import type { Registration } from "../protocol/registration";
import type { SystemInfo } from "../protocol/system";
import { recordLogReceipt } from "../logs";
import { registrationHasParent } from "../registry/registration";
import type { AnyModule, RegistrySlice, Slice } from "./types";

const factories = {
  Led: createLed,
  Button: createButton,
  Imu: createImu,
  Lidar: createLidar,
  Rangefinder: createRangefinder,
  RemoteReceiver: createRemoteReceiver,
  Rfid: createRfid,
  Servo: createServo,
  StepperMotor: createStepper,
};

export const createRegistrySlice: Slice<RegistrySlice> = (set, get) => ({
  ModuleRegistry: {},
  LookUpId: {},
  DuplicateLookUpIds: [],
  systemInfo: null,

  HandleEvent: (event: ProtocolMessage) => {
    recordLogReceipt(event);
    get().pushLog(event);
    try {
      if ("Registration" in event) get().RegistrationEvent(event.Registration);
      else if ("ModuleEvent" in event) get().ModuleEvent(event.ModuleEvent);
      else if ("System" in event) get().SystemEvent(event.System);
    } catch (error) {
      console.error("{HandleEvent} Failed to handle event:", error);
    }
  },

  RegistrationEvent: (event: Registration) => {
    try {
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
      const moduleStore = factory(registrationHasParent(event), event.id, event.lool_up_id) as StoreApi<AnyModule>;
      set((state) => ({
        ModuleRegistry: { ...state.ModuleRegistry, [event.id]: moduleStore },
        LookUpId: { ...state.LookUpId, [event.lool_up_id]: event.id },
      }));
    } catch (error) {
      console.error("{RegistrationEvent} Failed to handle registration event:", error);
    }
  },

  ModuleEvent: (data: EventPackage) => {
    try {
      // SysLogs are shown by the log history, whatever their source; the reserved
      // source "log" (contract C-05) has no module store (PIN-TPL-017, Phase 3A:
      // before, each one was reported as "Module store not found").
      if ("SysLog" in data.event) return;
      const moduleStore = get().ModuleRegistry[data.id];
      if (!moduleStore) {
        console.error("Module store not found for ID:", data.id);
        return;
      }
      // An event's family name is the same as the kind of store it belongs to.
      const [family, payload] = Object.entries(data.event)[0];
      const module = moduleStore.getState();
      if (family !== module.kind) {
        console.error("Module event kind mismatch:", data.id, family);
        return;
      }
      (module.handleEvent as (event: unknown) => void)(payload);
    } catch (error) {
      console.error("{ModuleEvent} Failed to handle module event:", error);
    }
  },

  SystemEvent: (event: SystemInfo) => {
    set({ systemInfo: event });
  },

  BuildViewCache: () =>
    Object.entries(get().LookUpId).map(([look_up_id, id]) => ({
      id,
      look_up_id,
      kind: get().ModuleRegistry[id]?.getState().kind ?? "",
    })),
});
