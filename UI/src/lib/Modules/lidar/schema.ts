import { z } from "zod";

export const RangePointSchema = z.object({
  x: z.number().int(),
  y: z.number().int(),
  distant: z.number(),
});

export type RangePoint = z.infer<typeof RangePointSchema>;

export const PointSchema = z.object({
  x: z.number().int().min(-90).max(90),
  y: z.number().int().min(-90).max(90),
});

export type Point = z.infer<typeof PointSchema>;

export const ScanStateSchema = z.enum(["Idol", "Scanning", "StopScan"]);

export type ScanState = z.infer<typeof ScanStateSchema>;

export const LidarEventSchema = z.union([
  z.object({
    Roi: z.object({
      min: PointSchema,
      max: PointSchema,
    }),
  }),

  z.object({
    PointMap: z.object({
      max_chunk: z.number(),
      curr_chunk: z.number(),
      map: z.array(RangePointSchema),
    }),
  }),

  z.object({
    Target: z.object({
      point: PointSchema,
    }),
  }),

  z.object({
    ScanState: z.object({
      state: ScanStateSchema,
      scan_time: z.number(),
    }),
  }),
]);

export type LidarEvent = z.infer<typeof LidarEventSchema>;

export const LidarCommandSchema = z.union([
  z.object({
    Roi: z.object({
      min: PointSchema,
      max: PointSchema,
    }),
  }),

  z.object({ StartScan: z.object({}) }),
  z.object({ StopScan: z.object({}) }),
  z.object({ Test: z.object({}) }),

  z.object({
    SetStep: z.object({
      step: z.number().int().min(1).max(180),
    }),
  }),

  z.object({
    ChangeMotorAngle: z.object({
      id: z.string(),
      step: z.number().int().min(-90).max(90),
    }),
  }),

  z.object({
    MovePos: z.object({
      p: PointSchema,
    }),
  }),
]);

export type LidarCommand = z.infer<typeof LidarCommandSchema>;
