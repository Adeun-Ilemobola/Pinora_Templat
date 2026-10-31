import { z } from "zod";

export const PivotPointSchema = z.enum(["Min", "Max"]);

export type PivotPoint = z.infer<typeof PivotPointSchema>;

export const StepperStateTypeSchema = z.enum([
  "Idle",
  "Moving",
  "Homing",
  "Pivot",
]);

export type StepperStateType = z.infer<typeof StepperStateTypeSchema>;

export const StepperMotorCommandSchema = z.union([
  z.object({
    SetPivotMin: z.object({
      pivot_min: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({
    SetPivotMax: z.object({
      pivot_max: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({ MoveToOrigin: z.object({}) }),

  z.object({
    MoveToAngle: z.object({
      angle: z.number().min(-3.4028234663852886e38).max(3.4028234663852886e38),
    }),
  }),

  z.object({ MoveToPivotMin: z.object({}) }),

  z.object({ MoveToPivotMax: z.object({}) }),

  z.object({
    SetMode: z.object({
      mode: StepperStateTypeSchema,
    }),
  }),
]);

export type StepperMotorCommand = z.infer<typeof StepperMotorCommandSchema>;

export const StepperMotorEventSchema = z.union([
  z.object({
    GetAngle: z.object({
      angle: z.number().min(-3.4028234663852886e38).max(3.4028234663852886e38),
      step: z.number().min(-3.4028234663852886e38).max(3.4028234663852886e38),
    }),
  }),

  z.object({
    GetPivotMin: z.object({
      pivot_min: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({
    GetPivotMax: z.object({
      pivot_max: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({
    GetMode: z.object({
      mode: StepperStateTypeSchema,
    }),
  }),

  z.object({
    GetOrigin: z.object({
      origin: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38)
        .nullable(),
    }),
  }),

  z.object({
    GetPivotPoint: z.object({
      pivot_point: PivotPointSchema,
    }),
  }),
]);

export type StepperMotorEvent = z.infer<typeof StepperMotorEventSchema>;
