import z from "zod";
import { LedCommandTypeSchema } from "../../mainview/Modules/led/definition"; 
import { LidarCommandTypeSchema } from "../../mainview/Modules/Lidar/definition";
import {
  RangefinderCommandTypeSchema,
  RangefinderDistanceModeSchema,
} from "../../mainview/Modules/rangefinder/definition";
import { ServoCommandTypeSchema } from "../../mainview/Modules/servo/definition"; 

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

