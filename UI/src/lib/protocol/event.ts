import { ButtonEventSchema } from "../Modules/button";
//
import { z } from "zod";
//import { LedEventSchema } from "../Modules/Led";

export const LogPrioritySchema = z.enum(["Low", "Medium", "High", "Critical"]);

export type LogPriority = z.infer<typeof LogPrioritySchema>;

export const SysLogEventSchema = z.strictObject({
  text: z.string(),
  raw_err: z.string().nullable(),
  priority: LogPrioritySchema,
});

export type SysLogEvent = z.infer<typeof SysLogEventSchema>;

export const ModuleEventSchema = z.union([
  z.strictObject({ Button: ButtonEventSchema }),
  

  // z.strictObject({
  //   Led: LedEventSchema,
  //}),

  // z.strictObject({
  //   Lidar: LidarEventSchema,
  // }),

  z.strictObject({
    SysLog: SysLogEventSchema,
  }),
]);

export type ModuleEvent = z.infer<typeof ModuleEventSchema>;

export const EventPackageSchema = z.strictObject({
  event: ModuleEventSchema,
  id: z.string(),
});

export type EventPackage = z.infer<typeof EventPackageSchema>;
