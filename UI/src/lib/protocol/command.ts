
import { z } from "zod";
//import { LedCommandSchema } from "../Modules/Led";

export const ModuleCommandSchema = z.union([
  // z.object({ Servo: ServoCommandSchema }),
  // z.object({ Rangefinder: RangefinderCommandSchema }),
  // z.object({ Rfid: RfidCommandSchema }),
  // z.object({ StepperMotor: StepperMotorCommandSchema }),

    // z.object({
    //   Led: LedCommandSchema,
    // }),

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
