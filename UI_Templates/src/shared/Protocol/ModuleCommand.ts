import z from "zod";
import { LedCommandTypeSchema } from "@modules/led/definition";
import { LidarCommandTypeSchema } from "@modules/Lidar/definition";
import {
  RangefinderCommandTypeSchema,
  RangefinderDistanceModeSchema,
} from "@modules/rangefinder/definition";
import { ServoCommandTypeSchema } from "@modules/servo/definition";

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
  // z.object({
  //   id: z.string(),
  //   module_type: z.literal("ClusterLeds"),
  //   payload: ClusterCommandTypeSchema,
  // }),
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

export type Commandtype = z.infer<typeof ModuleCommandSchema>;

export {
  // ClusterCommandTypeSchema,
  LedCommandTypeSchema,
  LidarCommandTypeSchema,
  RangefinderCommandTypeSchema,
  RangefinderDistanceModeSchema,
  ServoCommandTypeSchema,
};
