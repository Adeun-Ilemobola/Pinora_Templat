import { z } from "zod";

export const RangefinderDistanceModeSchema = z.enum(["Short", "Long"]);

export type RangefinderDistanceMode = z.infer<
  typeof RangefinderDistanceModeSchema
>;

export const RangefinderCommandSchema = z.union([
  z.literal("StartRanging"),

  z.literal("StopRanging"),

  z.object({
    SetTimingBudget: z.object({
      milliseconds: z.number().int().min(0).max(65535),
    }),
  }),

  z.object({
    SetDistanceMode: z.object({
      mode: RangefinderDistanceModeSchema,
    }),
  }),
]);

export type RangefinderCommand = z.infer<typeof RangefinderCommandSchema>;

export const RangefinderEventSchema = z.union([
  z.object({
    Range: z.object({
      millimeters: z.number().int().min(0).max(65535),
    }),
  }),

  z.object({
    RangingState: z.object({
      is_ranging: z.boolean(),
    }),
  }),

  z.object({
    TimingBudget: z.object({
      milliseconds: z.number().int().min(0).max(65535),
    }),
  }),

  z.object({
    DistanceMode: z.object({
      mode: RangefinderDistanceModeSchema,
    }),
  }),

  z.object({
    InvalidMeasurement: z.object({
      status: z.string(),
    }),
  }),
]);

export type RangefinderEvent = z.infer<typeof RangefinderEventSchema>;
