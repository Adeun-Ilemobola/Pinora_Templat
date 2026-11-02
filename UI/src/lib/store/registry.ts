// The module registry: what the device has told us (audit Phase 2D split).
//
// Protocol messages are routed by their top-level key. Only Led and Button
// registrations create stores. Re-sent registrations (rediscovery) keep the existing
// store; duplicate lookup names are recorded (PIN-TPL-017).
import type { StoreApi } from "zustand/vanilla";

import { createButton, type ButtonModule } from "../Modules/button";
import { createLed, type LedModule } from "../Modules/Led";
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
      const moduleStore = get().ModuleRegistry[data.id];
      if (!moduleStore) {
        console.error("Module store not found for ID:", data.id);
        return;
      }
      const event = data.event;
      if ("Led" in event && moduleStore.getState().kind === "Led") {
        (moduleStore as StoreApi<LedModule>).getState().handleEvent(event.Led);
      } else if ("Button" in event && moduleStore.getState().kind === "Button") {
        (moduleStore as StoreApi<ButtonModule>).getState().handleEvent(event.Button);
      } else {
        console.error("Module event kind mismatch:", data.id, Object.keys(event)[0]);
      }
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
