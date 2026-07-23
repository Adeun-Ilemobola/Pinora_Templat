import z from "zod";
import type { ModuleDefinitionType } from "../ModuleDefinitionSchema";

export const ServoCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("SetAngle"),
    angle: z.number(),
  }),
  z.object({
    command: z.literal("SetMinPivot"),
    min_pivot: z.number(),
  }),
  z.object({
    command: z.literal("SetMaxPivot"),
    max_pivot: z.number(),
  }),
]);

export const ServoEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("GetAngle"),
    id: z.string(),
    angle: z.number(),
  }),
  z.object({
    event_type: z.literal("GetMinPivot"),
    id: z.string(),
    min_pivot: z.number(),
  }),
  z.object({
    event_type: z.literal("GetMaxPivot"),
    id: z.string(),
    max_pivot: z.number(),
  }),
  z.object({
    event_type: z.literal("GetOffset"),
    id: z.string(),
    angle: z.number(),
  }),
]);

export const ServoModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: z.literal("Servo"),
  state: z.object({
    angle: z.number(),
  }),
});

export function servoInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): z.infer<typeof ServoModule> {
  return {
    id,
    parent_id,
    module_type: "Servo",
    lool_up_id,
    state: {
      angle: 0,
    },
  };
}

export function updateServo(
  module: ModuleDefinitionType,
  event: z.infer<typeof ServoEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Servo") return module;

  switch (event.event_type) {
    case "GetAngle":
      return {
        ...module,
        state: { angle: event.angle },
      };

    case "GetMinPivot":
    case "GetMaxPivot":
    case "GetOffset":
      return module;
  }
}
