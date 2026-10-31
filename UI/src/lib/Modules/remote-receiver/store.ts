import { createStore, type StoreApi } from "zustand/vanilla";
import { RemoteButton, RemoteButtonEvent } from "./schema";

type RemoteReceiverInstance = {
  hasParent: boolean;
  id: string;
  kind: "RemoteReceiver";
  look_up_id: string;
  state: { key: RemoteButton | null; clickCount: number };
};
export interface RemoteReceiverModule extends RemoteReceiverInstance {
  handleEvent: (event: RemoteButtonEvent) => void;
}
export function createRemoteReceiver(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<RemoteReceiverModule> {
  return createStore<RemoteReceiverModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    state: { key: null, clickCount: 0 },
    kind: "RemoteReceiver",
    handleEvent: (event) => {
      set({
        state: {
          ...get().state,
          key: event.Click.key,
          clickCount: get().state.clickCount + 1,
        },
      });
    },
  }));
}
