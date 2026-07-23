import z from "zod";
import type { ModuleDefinitionType } from "../ModuleDefinitionSchema";

export const ButtonEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Ckick"),
    id: z.string(),
  }),
]);

export const ButtonModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  module_type: z.literal("Button"),
  parent_id: z.string(),
  state: z.object({
    on: z.boolean(),
  }),
});

export function buttonInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): z.infer<typeof ButtonModule> {
  return {
    id,
    parent_id,
    module_type: "Button",
    lool_up_id,
    state: {
      on: false,
    },
  };
}

export function updateButton(
  module: ModuleDefinitionType,
  event: z.infer<typeof ButtonEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Button") return module;

  switch (event.event_type) {
    case "Ckick":
      return {
        ...module,
        state: { on: !module.state.on },
      };
  }
}
