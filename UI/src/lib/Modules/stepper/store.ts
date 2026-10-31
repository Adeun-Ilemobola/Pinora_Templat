import { createStore, type StoreApi } from "zustand/vanilla";
import { IncomingCommand } from "../../IncomingCommand";
import { PivotPoint, StepperStateType, StepperMotorCommandSchema, StepperMotorEvent } from "./schema";

type StepperInstance = {
  hasParent: boolean;
  id: string;
  kind: "StepperMotor";
  look_up_id: string;
  state: {
    GetAngle: { angle: number; step: number } | null;
    GetPivotMin: { pivot_min: number } | null;
    GetPivotMax: { pivot_max: number } | null;
    GetMode: { mode: StepperStateType } | null;
    GetOrigin: { origin: number | null } | null;
    GetPivotPoint: { pivot_point: PivotPoint } | null;
  };
};
export interface StepperModule extends StepperInstance {
  handleEvent: (event: StepperMotorEvent) => void;
  setPivotMin: (pivot_min: number) => Promise<unknown>;
  setPivotMax: (pivot_max: number) => Promise<unknown>;
  moveToOrigin: () => Promise<unknown>;
  moveToAngle: (angle: number) => Promise<unknown>;
  moveToPivotMin: () => Promise<unknown>;
  moveToPivotMax: () => Promise<unknown>;
  setMode: (mode: StepperStateType) => Promise<unknown>;
}
export function createStepper(hasParent: boolean, id: string, look_up_id: string): StoreApi<StepperModule> {
  return createStore<StepperModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    state: { GetAngle: null, GetPivotMin: null, GetPivotMax: null, GetMode: null, GetOrigin: null, GetPivotPoint: null },
    kind: "StepperMotor",
    setPivotMin: (pivot_min: number) =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({
            SetPivotMin: { pivot_min },
          }),
        },
      }),
    setPivotMax: (pivot_max: number) =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({
            SetPivotMax: { pivot_max },
          }),
        },
      }),
    moveToOrigin: () =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ MoveToOrigin: {} }),
        },
      }),
    moveToAngle: (angle: number) =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({
            MoveToAngle: { angle },
          }),
        },
      }),
    moveToPivotMin: () =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ MoveToPivotMin: {} }),
        },
      }),
    moveToPivotMax: () =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ MoveToPivotMax: {} }),
        },
      }),
    setMode: (mode: StepperStateType) =>
      IncomingCommand({
        id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ SetMode: { mode } }),
        },
      }),
    handleEvent: (event) => {
      if ("GetAngle" in event)
        set({ state: { ...get().state, GetAngle: event.GetAngle } });
      if ("GetPivotMin" in event)
        set({ state: { ...get().state, GetPivotMin: event.GetPivotMin } });
      if ("GetPivotMax" in event)
        set({ state: { ...get().state, GetPivotMax: event.GetPivotMax } });
      if ("GetMode" in event)
        set({ state: { ...get().state, GetMode: event.GetMode } });
      if ("GetOrigin" in event)
        set({ state: { ...get().state, GetOrigin: event.GetOrigin } });
      if ("GetPivotPoint" in event)
        set({ state: { ...get().state, GetPivotPoint: event.GetPivotPoint } });
    },
  }));
}
