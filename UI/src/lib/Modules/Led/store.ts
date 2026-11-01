import { createStore, StoreApi } from "zustand/vanilla";
import { IncomingCommand } from "../../IncomingCommand";
import { LedEvent } from "./schema";

type LedInstance = {
  id: string;
  kind: "Led";
  /** Reported brightness; `null` until the device reports it (Phase 2A, PIN-TPL-012). */
  state: number | null;
  look_up_id: string;
  hasParent: boolean;
};

export interface LedModule extends LedInstance {
  toggle: () => void;
  setBrightness: (v: number) => void;
  handleEvent: (event: any) => void;
}

export function createLed(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<LedModule> {
  return createStore<LedModule>((set) => ({
    hasParent,
    id,
    kind: "Led",
    state: null,
    look_up_id,
    toggle: () => {
      const payload = {
        id,
        command: {
          Led: {
            Toggle: {},
          },
        },
      };
      IncomingCommand(payload);
    },
    setBrightness: (v) => {
      const payload = {
        id,
        command: {
          Led: {
            SetState: { state: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    handleEvent: (event: LedEvent) => {
      const [key, value] = Object.entries(event)[0];
      // Handle incoming events for the LED module

      if (key === "Brightness") {
        console.log("[LedModule - CORE] Brightness level:", value.level);
        set({ state: value.level });
      }
    },
  }));
}
