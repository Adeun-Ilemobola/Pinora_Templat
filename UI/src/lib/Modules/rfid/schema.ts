import { z } from "zod";

export const RfidModeSchema = z.enum(["Read", "Write"]);

export type RfidMode = z.infer<typeof RfidModeSchema>;

export const RfidWriteStateSchema = z.enum(["Good", "Bad"]);

export type RfidWriteState = z.infer<typeof RfidWriteStateSchema>;

export const RfidCommandSchema = z.union([
  z.object({ WriteMode: z.object({}) }),

  z.object({ ReadMode: z.object({}) }),

  z.object({
    WritePayload: z.object({
      data: z.array(z.number().int().min(0).max(255)),
    }),
  }),
]);

export type RfidCommand = z.infer<typeof RfidCommandSchema>;

export const RfidEventSchema = z.union([
  z.object({
    GetCard: z.object({
      card_uid: z.string(),
      card_data: z.string(),
    }),
  }),

  z.object({
    GetMode: z.object({
      mode: RfidModeSchema,
    }),
  }),

  z.object({
    GetWriteState: z.object({
      state: RfidWriteStateSchema,
      info: z.string(),
    }),
  }),
]);

export type RfidEvent = z.infer<typeof RfidEventSchema>;
