import { createStore, type StoreApi } from "zustand/vanilla";
import { ButtonEvent } from "./schema";

type ButtonInstance = {
  hasParent: boolean;
  id: string;
  kind: "Button";
  look_up_id: string;
  state: { clickCount: number };
};
export interface ButtonModule extends ButtonInstance {
  handleEvent: (event: ButtonEvent) => void;
}
export function createButton(hasParent: boolean , id: string, look_up_id: string): StoreApi<ButtonModule> {
  return createStore<ButtonModule>((set, get) => ({
    hasParent,
    id,
    kind: "Button",
    look_up_id,
    state: { clickCount: 0 },
    handleEvent: (event) => {
      if ("Ckick" in event)
        set({
          state: { ...get().state, clickCount: get().state.clickCount + 1 },
        });
    },
  }));
}
