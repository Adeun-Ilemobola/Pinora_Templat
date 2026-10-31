import { createStore, type StoreApi } from "zustand/vanilla";
import { IncomingCommand } from "../../IncomingCommand";
import { RfidMode, RfidWriteState, RfidCommandSchema, RfidEvent } from "./schema";

type RfidInstance = {
  hasParent: boolean;
  id: string;
  kind: "Rfid";
  look_up_id: string;
  state: {
    GetCard: { card_uid: string; card_data: string } | null;
    GetMode: { mode: RfidMode } | null;
    GetWriteState: { state: RfidWriteState; info: string } | null;
  };
};
export interface RfidModule extends RfidInstance {
  handleEvent: (event: RfidEvent) => void;
  readMode: () => Promise<unknown>;
  writeMode: () => Promise<unknown>;
  writePayload: (data: number[]) => Promise<unknown>;
}
export function createRfid(hasParent: boolean, id: string, look_up_id: string): StoreApi<RfidModule> {
  return createStore<RfidModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    state: { GetCard: null, GetMode: null, GetWriteState: null },
    kind: "Rfid",
    readMode: () =>
      IncomingCommand({
        id,
        command: { Rfid: RfidCommandSchema.parse({ ReadMode: {} }) },
      }),
    writeMode: () =>
      IncomingCommand({
        id,
        command: { Rfid: RfidCommandSchema.parse({ WriteMode: {} }) },
      }),
    writePayload: (bytes: number[]) =>
      IncomingCommand({
        id,
        command: {
          Rfid: RfidCommandSchema.parse({ WritePayload: { data: bytes } }),
        },
      }),
    handleEvent: (event) => {
      if ("GetCard" in event)
        set({ state: { ...get().state, GetCard: event.GetCard } });
      if ("GetMode" in event)
        set({ state: { ...get().state, GetMode: event.GetMode } });
      if ("GetWriteState" in event)
        set({ state: { ...get().state, GetWriteState: event.GetWriteState } });
    },
  }));
}
