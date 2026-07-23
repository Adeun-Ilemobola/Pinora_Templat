import z from "zod";
import type { ModuleDefinitionType } from "../ModuleDefinitionSchema";

export const LedCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("SetState"),
    state: z.number(),
  }),
  z.object({
    command: z.literal("Toggle"),
  }),
]);
export const LedEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Brightness"),
    id: z.string(),
    level: z.number(),
  }),
]);


export const LedModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  module_type: z.literal("Led"),
  parent_id: z.string(),
  state: z.object({
    brightness: z.number(),
  }),
});

export function ledInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): z.infer<typeof LedModule> {
  return {
    id,
    parent_id,
    module_type: "Led",
    lool_up_id,
    state: {
      brightness: 0,
    },
  };
}

export function updateLed(
  module: ModuleDefinitionType,
  event: z.infer<typeof LedEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Led") return module;

  switch (event.event_type) {
    case "Brightness":
      return {
        ...module,
        state: { brightness: event.level },
      };
  }
}
