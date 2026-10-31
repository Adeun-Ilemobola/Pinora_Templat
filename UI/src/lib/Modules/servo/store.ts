import { StoreApi, createStore } from "zustand/vanilla";
import { IncomingCommand } from "../../IncomingCommand";
import { ServoEvent } from "./schema";

type ServoInstance = {
  hasParent: boolean;
  id: string;
  kind: "Servo";
  look_up_id: string;
  state: {
    MinPivot: number;
    MaxPivot: number;
    Offset: number;
    Angle: number;
  };
};

export interface ServoModule extends ServoInstance {
  setAngle: (v: number) => void;
  setMinPivot: (v: number) => void;
  setMaxPivot: (v: number) => void;
  handleEvent: (event: any) => void;
}

export function createServo(hasParent: boolean, id: string, look_up_id: string): StoreApi<ServoModule> {
  return createStore<ServoModule>((set, get) => ({
    hasParent,
    kind: "Servo",
    state: {
      MinPivot: 0,
      MaxPivot: 0,
      Offset: 0,
      Angle: 0,
    },
    look_up_id,
    id,

    setAngle: (v) => {
      const payload = {
        id,
        command: {
          Servo: {
            SetAngle: { angle: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    setMinPivot: (v) => {
      const payload = {
        id,
        command: {
          Servo: {
            SetMinPivot: { min_pivot: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    setMaxPivot: (v) => {
      const payload = {
        id,
        command: {
          Servo: {
            SetMaxPivot: { max_pivot: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    // setOffset: (v) => {
    //   const payload = {
    //     id,
    //     command: {
    //       Servo: {
    //         SetOffset: { offset: v },
    //       },
    //     },
    //   };
    //   IncomingCommand(payload);
    // },
    handleEvent: (event: ServoEvent) => {
      // Handle incoming events for the Servo module
      const [key, value] = Object.entries(event)[0];

      if (key === "GetAngle") {
        set({ state: { ...get().state, Angle: value.angle } });
      }
      if (key === "GetMinPivot") {
        set({ state: { ...get().state, MinPivot: value.min_pivot } });
      }
      if (key === "GetMaxPivot") {
        set({ state: { ...get().state, MaxPivot: value.max_pivot } });
      }
      if (key === "GetOffset") {
        set({ state: { ...get().state, Offset: value.angle } });
      }
    },
  }));
}
