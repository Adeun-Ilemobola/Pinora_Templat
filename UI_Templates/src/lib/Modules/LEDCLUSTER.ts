import z from "zod";

// The backend currently exposes commands for LED clusters, but does not yet
// expose a cluster state definition or event payload.
export const ClusterCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("ToggleAll"),
  }),
  z.object({
    command: z.literal("SetAll"),
    state: z.number(),
  }),
  z.object({
    command: z.literal("Toggle"),
    id: z.string(),
    state: z.number(),
  }),
  z.object({
    command: z.literal("SetState"),
    id: z.string(),
    state: z.number(),
  }),
]);
