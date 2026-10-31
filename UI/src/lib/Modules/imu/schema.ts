import { z } from "zod";

export const RawAxesSchema = z.object({
  x: z.number().int().min(-32768).max(32767),
  y: z.number().int().min(-32768).max(32767),
  z: z.number().int().min(-32768).max(32767),
});

export type RawAxes = z.infer<typeof RawAxesSchema>;

export const AxesSchema = z.object({
  x: z.number(),
  y: z.number(),
  z: z.number(),
});

export type Axes = z.infer<typeof AxesSchema>;

export const MpuDeviceModeSchema = z.enum(["Collecting", "Idle", "Off"]);

export type MpuDeviceMode = z.infer<typeof MpuDeviceModeSchema>;

export const ImuEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Gyro"),
    raw_axes: RawAxesSchema,
    axes: AxesSchema,
  }),

  z.object({
    event_type: z.literal("Accel"),
    raw_axes: RawAxesSchema,
    axes: AxesSchema,
  }),

  z.object({
    event_type: z.literal("Mode"),
    mode: MpuDeviceModeSchema,
  }),
]);

export type ImuEvent = z.infer<typeof ImuEventSchema>;
