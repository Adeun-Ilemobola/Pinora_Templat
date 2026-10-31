import { z } from "zod";

export const ButtonEventSchema = z.union([
  z.object({
    Ckick: z.object({}),
  }),
]);

export type ButtonEvent = z.infer<typeof ButtonEventSchema>;
