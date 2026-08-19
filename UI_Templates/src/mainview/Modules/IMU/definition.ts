import z from "zod";
import type { ModuleDefinitionType } from "@src/bun/Protocol/ModuleDefinitionSchema";

export const ImuAxesSchema = z.object({
  x: z.number(),
  y: z.number(),
  z: z.number(),
});

export const ImuRawAxesSchema = z.object({
  x: z.number().int(),
  y: z.number().int(),
  z: z.number().int(),
});

export const ImuModeSchema = z.enum(["Collecting", "Idle", "Off"]);

export const ImuEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Gyro"),
    id: z.string(),
    raw_axes: ImuRawAxesSchema,
    axes: ImuAxesSchema,
  }),
  z.object({
    event_type: z.literal("Accel"),
    id: z.string(),
    raw_axes: ImuRawAxesSchema,
    axes: ImuAxesSchema,
  }),
  z.object({
    event_type: z.literal("Mode"),
    // The current firmware broadcasts mode events without an id.
    id: z.string().optional(),
    mode: ImuModeSchema,
  }),
]);

export const ImuModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: z.literal("Imu"),
  state: z.object({
    mode: ImuModeSchema,
    gyro: ImuAxesSchema,
    gyro_raw: ImuRawAxesSchema,
    accel: ImuAxesSchema,
    accel_raw: ImuRawAxesSchema,
  }),
  mutableStateFields: z.tuple([]),
});

export type ImuAxes = z.infer<typeof ImuAxesSchema>;
export type ImuMode = z.infer<typeof ImuModeSchema>;
export type ImuDefinition = z.infer<typeof ImuModule>;

const ZERO_AXES: ImuAxes = { x: 0, y: 0, z: 0 };

export function imuInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): ImuDefinition {
  return {
    id,
    parent_id,
    lool_up_id,
    module_type: "Imu",
    state: {
      mode: "Collecting",
      gyro: { ...ZERO_AXES },
      gyro_raw: { ...ZERO_AXES },
      accel: { ...ZERO_AXES },
      accel_raw: { ...ZERO_AXES },
    },
    mutableStateFields: [],
  };
}

export function updateImu(
  module: ModuleDefinitionType,
  event: z.infer<typeof ImuEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Imu") return module;

  switch (event.event_type) {
    case "Gyro":
      return {
        ...module,
        state: {
          ...module.state,
          gyro: event.axes,
          gyro_raw: event.raw_axes,
        },
      };

    case "Accel":
      return {
        ...module,
        state: {
          ...module.state,
          accel: event.axes,
          accel_raw: event.raw_axes,
        },
      };

    case "Mode":
      return {
        ...module,
        state: { ...module.state, mode: event.mode },
      };
  }
}
