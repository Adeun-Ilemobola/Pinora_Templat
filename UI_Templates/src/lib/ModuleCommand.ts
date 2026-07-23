import z from "zod";
import { LedCommandTypeSchema } from "./Modules/LED";
import { ClusterCommandTypeSchema } from "./Modules/LEDCLUSTER";
import { LidarCommandTypeSchema } from "./Modules/LIDAR";
import {
  RangefinderCommandTypeSchema,
  RangefinderDistanceModeSchema,
} from "./Modules/RANGEFINDER";
import { ServoCommandTypeSchema } from "./Modules/SERVO";

export const ModuleCommandSchema = z.discriminatedUnion("module_type", [
  z.object({
    id: z.string(),
    module_type: z.literal("Led"),
    payload: LedCommandTypeSchema,
  }),
  z.object({
    id: z.string(),
    module_type: z.literal("Servo"),
    payload: ServoCommandTypeSchema,
  }),
  z.object({
    id: z.string(),
    module_type: z.literal("ClusterLeds"),
    payload: ClusterCommandTypeSchema,
  }),
  z.object({
    id: z.string(),
    module_type: z.literal("Lidar"),
    payload: LidarCommandTypeSchema,
  }),
  z.object({
    id: z.string(),
    module_type: z.literal("Rangefinder"),
    payload: RangefinderCommandTypeSchema,
  }),
]);

export type Command = z.infer<typeof ModuleCommandSchema>;

export {
  ClusterCommandTypeSchema,
  LedCommandTypeSchema,
  LidarCommandTypeSchema,
  RangefinderCommandTypeSchema,
  RangefinderDistanceModeSchema,
  ServoCommandTypeSchema,
};

export type { RangefinderDistanceMode } from "./Modules/RANGEFINDER";
