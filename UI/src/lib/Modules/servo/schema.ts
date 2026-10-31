import { z } from "zod";

export const ServoCapabilitySchema = z.object({
  max_angle: z.number(),
  min_angle: z.number(),
  offset: z.number(),
  min_pivot: z.number(),
  max_pivot: z.number(),
  pulse_min: z.number(),
  pulse_max: z.number(),
});

export type ServoCapability = z.infer<typeof ServoCapabilitySchema>;

export const ServoEventSchema = z.union([
  z.object({
    GetAngle: z.object({
      angle: z.number(),
    }),
  }),

  z.object({
    GetMinPivot: z.object({
      min_pivot: z.number(),
    }),
  }),

  z.object({
    GetMaxPivot: z.object({
      max_pivot: z.number(),
    }),
  }),

  z.object({
    GetOffset: z.object({
      angle: z.number(),
    }),
  }),
]);

export type ServoEvent = z.infer<typeof ServoEventSchema>;

export const ServoCommandSchema = z.union([
  z.object({
    SetAngle: z.object({
      angle: z.number(),
    }),
  }),

  z.object({
    SetMinPivot: z.object({
      min_pivot: z.number(),
    }),
  }),

  z.object({
    SetMaxPivot: z.object({
      max_pivot: z.number(),
    }),
  }),
]);

export type ServoCommand = z.infer<typeof ServoCommandSchema>;
