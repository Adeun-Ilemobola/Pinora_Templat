import { createStore, type StoreApi } from "zustand/vanilla";
import { MpuDeviceMode, ImuEvent } from "./schema";

type ImuInstance = {
  hasParent: boolean;
  id: string;
  kind: "Imu";
  look_up_id: string;
  state: {
    Gyro: Extract<ImuEvent, { event_type: "Gyro" }> | null;
    Accel: Extract<ImuEvent, { event_type: "Accel" }> | null;
    Mode: MpuDeviceMode | null;
  };
};
export interface ImuModule extends ImuInstance {
  handleEvent: (event: ImuEvent) => void;
}
export function createImu(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<ImuModule> {
  return createStore<ImuModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    kind: "Imu",
    state: {
      Gyro: null,
      Accel: null,
      Mode: null,
    },
    handleEvent: (event) => {
      switch (event.event_type) {
        case "Gyro":
          set({ state: { ...get().state, Gyro: event } });
          break;
        case "Accel":
          set({ state: { ...get().state, Accel: event } });
          break;
        case "Mode":
          set({ state: { ...get().state, Mode: event.mode } });
          break;
      }
    },
  }));
}
