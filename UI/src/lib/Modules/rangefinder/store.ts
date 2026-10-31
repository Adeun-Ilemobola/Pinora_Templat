import { createStore, type StoreApi } from "zustand/vanilla";
import { IncomingCommand } from "../../IncomingCommand";
import { RangefinderDistanceMode, RangefinderCommandSchema, RangefinderEvent } from "./schema";

type RangefinderInstance = {
  hasParent: boolean;
  id: string;
  kind: "Rangefinder";
  look_up_id: string;
  state: {
    Range: { millimeters: number } | null;
    RangingState: { is_ranging: boolean } | null;
    TimingBudget: { milliseconds: number } | null;
    DistanceMode: { mode: RangefinderDistanceMode } | null;
    InvalidMeasurement: { status: string } | null;
  };
};
export interface RangefinderModule extends RangefinderInstance {
  handleEvent: (event: RangefinderEvent) => void;
  startRanging: () => Promise<unknown>;
  stopRanging: () => Promise<unknown>;
  setTimingBudget: (milliseconds: number) => Promise<unknown>;
  setDistanceMode: (mode: RangefinderDistanceMode) => Promise<unknown>;
}
export function createRangefinder(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<RangefinderModule> {
  return createStore<RangefinderModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    kind: "Rangefinder",
    state: {
      Range: null,
      RangingState: null,
      TimingBudget: null,
      DistanceMode: null,
      InvalidMeasurement: null,
    },
    startRanging: () =>
      IncomingCommand({
        id,
        command: {
          Rangefinder: RangefinderCommandSchema.parse("StartRanging"),
        },
      }),
    stopRanging: () =>
      IncomingCommand({
        id,
        command: { Rangefinder: RangefinderCommandSchema.parse("StopRanging") },
      }),
    setTimingBudget: (milliseconds: number) =>
      IncomingCommand({
        id,
        command: {
          Rangefinder: RangefinderCommandSchema.parse({
            SetTimingBudget: { milliseconds },
          }),
        },
      }),
    setDistanceMode: (mode: RangefinderDistanceMode) =>
      IncomingCommand({
        id,
        command: {
          Rangefinder: RangefinderCommandSchema.parse({
            SetDistanceMode: { mode },
          }),
        },
      }),
    handleEvent: (event) => {
      if ("Range" in event)
        set({ state: { ...get().state, Range: event.Range } });
      if ("RangingState" in event)
        set({ state: { ...get().state, RangingState: event.RangingState } });
      if ("TimingBudget" in event)
        set({ state: { ...get().state, TimingBudget: event.TimingBudget } });
      if ("DistanceMode" in event)
        set({ state: { ...get().state, DistanceMode: event.DistanceMode } });
      if ("InvalidMeasurement" in event)
        set({
          state: {
            ...get().state,
            InvalidMeasurement: event.InvalidMeasurement,
          },
        });
    },
  }));
}
