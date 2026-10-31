import { z } from "zod";

export const LedEventSchema = z.union([
  z.strictObject({
    Brightness: z.strictObject({
      level: z.number(),
    }),
  }),
]);

export type LedEvent = z.infer<typeof LedEventSchema>;

export const LedCommandSchema = z.union([
  z.strictObject({
    SetState: z.strictObject({
      state: z.number(),
    }),
  }),

  z.strictObject({
    Toggle: z.strictObject({}),
  }),
]);

export type LedCommand = z.infer<typeof LedCommandSchema>;
