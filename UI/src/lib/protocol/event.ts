// Zod schemas for module events (`EventPackage` in pinora-protocol).
//
// Every family is covered since Phase 2A (PIN-TPL-007); enforced against the golden
// fixture by UI/tests/protocol-fixtures.test.ts. Family schemas are imported from
// `../Modules/<family>/schema` directly to avoid an import cycle through the stores.
import { z } from "zod";
import { ButtonEventSchema } from "../Modules/button/schema";
import { ImuEventSchema } from "../Modules/imu/schema";
import { LedEventSchema } from "../Modules/Led/schema";
import { LidarEventSchema } from "../Modules/lidar/schema";
import { RangefinderEventSchema } from "../Modules/rangefinder/schema";
import { RemoteButtonEventSchema } from "../Modules/remote-receiver/schema";
import { RfidEventSchema } from "../Modules/rfid/schema";
import { ServoEventSchema } from "../Modules/servo/schema";
import { StepperMotorEventSchema } from "../Modules/stepper/schema";

export const LogPrioritySchema = z.enum(["Low", "Medium", "High", "Critical"]);

export type LogPriority = z.infer<typeof LogPrioritySchema>;

export const SysLogEventSchema = z.strictObject({
  text: z.string(),
  raw_err: z.string().nullable(),
  priority: LogPrioritySchema,
});

export type SysLogEvent = z.infer<typeof SysLogEventSchema>;

export const ModuleEventSchema = z.union([
  z.strictObject({ Led: LedEventSchema }),
  z.strictObject({ Servo: ServoEventSchema }),
  z.strictObject({ Lidar: LidarEventSchema }),
  z.strictObject({ Button: ButtonEventSchema }),
  z.strictObject({ SysLog: SysLogEventSchema }),
  z.strictObject({ Rangefinder: RangefinderEventSchema }),
  z.strictObject({ StepperMotor: StepperMotorEventSchema }),
  z.strictObject({ Imu: ImuEventSchema }),
  z.strictObject({ Rfid: RfidEventSchema }),
  z.strictObject({ RemoteReceiver: RemoteButtonEventSchema }),
]);

export type ModuleEvent = z.infer<typeof ModuleEventSchema>;

export const EventPackageSchema = z.strictObject({
  event: ModuleEventSchema,
  id: z.string(),
});

export type EventPackage = z.infer<typeof EventPackageSchema>;
