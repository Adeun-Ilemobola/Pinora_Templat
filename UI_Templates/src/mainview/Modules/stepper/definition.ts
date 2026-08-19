import z from "zod";
import type { ModuleDefinitionType } from "@src/bun/Protocol/ModuleDefinitionSchema";

export const StepperStateSchema = z.enum([
  "Idle",
  "Moving",
  "Homing",
  "Pivot",
]);

export const PivotPointSchema = z.enum(["Min", "Max"]);

export const StepperMotorCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("SetPivotMin"),
    pivot_min: z.number(),
  }),
  z.object({
    command: z.literal("SetPivotMax"),
    pivot_max: z.number(),
  }),
  z.object({
    command: z.literal("MoveToOrigin"),
  }),
  z.object({
    command: z.literal("MoveToAngle"),
    angle: z.number(),
  }),
  z.object({
    command: z.literal("MoveToPivotMin"),
  }),
  z.object({
    command: z.literal("MoveToPivotMax"),
  }),
  z.object({
    command: z.literal("SetMode"),
    mode: StepperStateSchema,
  }),
]);

export const StepperMotorEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("GetAngle"),
    id: z.string(),
    angle: z.number(),
    step: z.number(),
  }),
  z.object({
    event_type: z.literal("GetPivotMin"),
    id: z.string(),
    pivot_min: z.number(),
  }),
  z.object({
    event_type: z.literal("GetPivotMax"),
    id: z.string(),
    pivot_max: z.number(),
  }),
  z.object({
    event_type: z.literal("GetMode"),
    id: z.string(),
    mode: StepperStateSchema,
  }),
  z.object({
    event_type: z.literal("GetOrigin"),
    id: z.string(),
    origin: z.number().nullable(),
  }),
  z.object({
    event_type: z.literal("GetPivotPoint"),
    id: z.string(),
    pivot_point: PivotPointSchema,
  }),
]);

export const StepperMotorModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: z.literal("StepperMotor"),
  state: z.object({
    angle: z.number(),
    step: z.number(),
    pivot_min: z.number(),
    pivot_max: z.number(),
    mode: StepperStateSchema,
    origin: z.number().nullable(),
    pivot_point: PivotPointSchema,
  }),
  mutableStateFields: z.tuple([]),
});

export type StepperState = z.infer<typeof StepperStateSchema>;
export type PivotPoint = z.infer<typeof PivotPointSchema>;
export type StepperMotorDefinition = z.infer<typeof StepperMotorModule>;

export function stepperMotorInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): StepperMotorDefinition {
  return {
    id,
    parent_id,
    module_type: "StepperMotor",
    lool_up_id,
    state: {
      angle: 0,
      step: 0,
      pivot_min: -90,
      pivot_max: 90,
      mode: "Idle",
      origin: null,
      pivot_point: "Max",
    },
    mutableStateFields: [],
  };
}

export function updateStepperMotor(
  module: ModuleDefinitionType,
  event: z.infer<typeof StepperMotorEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "StepperMotor") return module;

  switch (event.event_type) {
    case "GetAngle":
      return {
        ...module,
        state: {
          ...module.state,
          angle: event.angle,
          step: event.step,
        },
      };

    case "GetPivotMin":
      return {
        ...module,
        state: { ...module.state, pivot_min: event.pivot_min },
      };

    case "GetPivotMax":
      return {
        ...module,
        state: { ...module.state, pivot_max: event.pivot_max },
      };

    case "GetMode":
      return {
        ...module,
        state: { ...module.state, mode: event.mode },
      };

    case "GetOrigin":
      return {
        ...module,
        state: { ...module.state, origin: event.origin },
      };

    case "GetPivotPoint":
      return {
        ...module,
        state: { ...module.state, pivot_point: event.pivot_point },
      };
  }
}
