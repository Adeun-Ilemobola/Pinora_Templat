import z from "zod";
import { ButtonEventSchema } from "../../mainview/Modules/button/definition";
import { LedEventSchema } from "../../mainview/Modules/led/definition";
import { LidarEventSchema ,PointSchema } from "../../mainview/Modules/Lidar/definition"; 
import { RangefinderEventSchema  } from "../../mainview/Modules/rangefinder/definition"; 
import { ServoEventSchema } from "../../mainview/Modules/servo/definition"; 

const SysLogEventSchema = z.object({
  text: z.string(),
  raw_err: z.string().nullable(),
  priority: z.enum(["Low", "Medium", "High", "Critical"]),
});





export const ModuleEventSchema = z.discriminatedUnion("module_type", [
  z.object({ module_type: z.literal("Led"), event: LedEventSchema }),
  z.object({ module_type: z.literal("Servo"), event: ServoEventSchema }),
  z.object({ module_type: z.literal("Lidar"), event: LidarEventSchema }),
  z.object({ module_type: z.literal("Button"), event: ButtonEventSchema }),
  z.object({ module_type: z.literal("SysLog"), event: SysLogEventSchema }),
  z.object({
    module_type: z.literal("Rangefinder"),
    event: RangefinderEventSchema,
  }),
  
]);

export type ModuleEventEnvelope = z.infer<typeof ModuleEventSchema>;

export {
  ButtonEventSchema,
  LedEventSchema,
  LidarEventSchema,
  PointSchema,
  RangefinderEventSchema,
  ServoEventSchema,
  SysLogEventSchema,
};
