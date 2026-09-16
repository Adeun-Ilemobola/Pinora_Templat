//import { ServoEventSchema } from "../Modules/servo";
import { ButtonEventSchema } from "../Modules/button";
// import { ImuEventSchema } from "../Modules/imu";
// import { RemoteButtonEventSchema } from "../Modules/remote-receiver";
// import { RangefinderEventSchema } from "../Modules/rangefinder";
// import { RfidEventSchema } from "../Modules/rfid";
// import { StepperMotorEventSchema } from "../Modules/stepper";
import { z } from "zod";
import { LedEventSchema } from "../Modules/Led";
//import { LidarEventSchema } from "../Modules/lidar";

export const LogPrioritySchema = z.enum(["Low", "Medium", "High", "Critical"]);

export type LogPriority = z.infer<typeof LogPrioritySchema>;

export const SysLogEventSchema = z.strictObject({
  text: z.string(),
  raw_err: z.string().nullable(),
  priority: LogPrioritySchema,
});

export type SysLogEvent = z.infer<typeof SysLogEventSchema>;

export const ModuleEventSchema = z.union([
  //z.strictObject({ Servo: ServoEventSchema }),
  z.strictObject({ Button: ButtonEventSchema }),
  //z.strictObject({ Imu: ImuEventSchema }),
  // z.strictObject({ RemoteReceiver: RemoteButtonEventSchema }),
  // z.strictObject({ Rangefinder: RangefinderEventSchema }),
  // z.strictObject({ Rfid: RfidEventSchema }),
  // z.strictObject({ StepperMotor: StepperMotorEventSchema }),

  z.strictObject({
    Led: LedEventSchema,
  }),

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
