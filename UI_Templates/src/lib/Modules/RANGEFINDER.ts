import z from "zod";
import type { ModuleDefinitionType } from "../ModuleDefinitionSchema";

export const RangefinderDistanceModeSchema = z.enum(["Short", "Long"]);

export type RangefinderDistanceMode = z.infer<
  typeof RangefinderDistanceModeSchema
>;

export const RangefinderCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("StartRanging"),
  }),
  z.object({
    command: z.literal("StopRanging"),
  }),
  z.object({
    command: z.literal("SetTimingBudget"),
    milliseconds: z.number().int().positive(),
  }),
  z.object({
    command: z.literal("SetDistanceMode"),
    mode: RangefinderDistanceModeSchema,
  }),
]);

export const RangefinderEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Range"),
    id: z.string(),
    millimeters: z.number().int().nonnegative(),
  }),
  z.object({
    event_type: z.literal("RangingState"),
    id: z.string(),
    is_ranging: z.boolean(),
  }),
  z.object({
    event_type: z.literal("TimingBudget"),
    id: z.string(),
    milliseconds: z.number().int().nonnegative(),
  }),
  z.object({
    event_type: z.literal("DistanceMode"),
    id: z.string(),
    mode: RangefinderDistanceModeSchema,
  }),
  z.object({
    event_type: z.literal("InvalidMeasurement"),
    id: z.string(),
    status: z.string(),
  }),
]);

export const RangefinderModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: z.literal("Rangefinder"),
  state: z.object({
    range_mm: z.number().int().nonnegative(),
    is_ranging: z.boolean(),
    timing_budget_ms: z.number().int().nonnegative(),
    distance_mode: RangefinderDistanceModeSchema,
    last_invalid_status: z.string().nullable(),
  }),
});

export function rangefinderInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): z.infer<typeof RangefinderModule> {
  return {
    id,
    parent_id,
    module_type: "Rangefinder",
    lool_up_id,
    state: {
      range_mm: 0,
      is_ranging: false,
      timing_budget_ms: 50,
      distance_mode: "Long",
      last_invalid_status: null,
    },
  };
}

export function updateRangefinder(
  module: ModuleDefinitionType,
  event: z.infer<typeof RangefinderEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Rangefinder") return module;

  switch (event.event_type) {
    case "Range":
      return {
        ...module,
        state: {
          ...module.state,
          range_mm: event.millimeters,
          last_invalid_status: null,
        },
      };

    case "RangingState":
      return {
        ...module,
        state: { ...module.state, is_ranging: event.is_ranging },
      };

    case "TimingBudget":
      return {
        ...module,
        state: { ...module.state, timing_budget_ms: event.milliseconds },
      };

    case "DistanceMode":
      return {
        ...module,
        state: { ...module.state, distance_mode: event.mode },
      };

    case "InvalidMeasurement":
      return {
        ...module,
        state: { ...module.state, last_invalid_status: event.status },
      };
  }
}
