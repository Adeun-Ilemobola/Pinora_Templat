//import { ServoCommandSchema } from "../Modules/servo";
//import { RangefinderCommandSchema } from "../Modules/rangefinder";
//import { RfidCommandSchema } from "../Modules/rfid";
//import { StepperMotorCommandSchema } from "../Modules/stepper";
import { z } from "zod";
import { LedCommandSchema } from "../Modules/Led";
//import { LidarCommandSchema } from "../Modules/lidar";

export const ModuleCommandSchema = z.union([
  // z.object({ Servo: ServoCommandSchema }),
  // z.object({ Rangefinder: RangefinderCommandSchema }),
  // z.object({ Rfid: RfidCommandSchema }),
  // z.object({ StepperMotor: StepperMotorCommandSchema }),

  z.object({
    Led: LedCommandSchema,
  }),

  // z.object({
  //   Lidar: LidarCommandSchema,
  // }),
]);

export type ModuleCommand = z.infer<typeof ModuleCommandSchema>;

export const IncomingCommandSchema = z.object({
  id: z.string(),
  command: ModuleCommandSchema,
});

export type IncomingCommand = z.infer<typeof IncomingCommandSchema>;
