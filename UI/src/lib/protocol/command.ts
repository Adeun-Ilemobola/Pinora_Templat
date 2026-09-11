import { z } from "zod";
import { LedCommandSchema } from "../Modules/Led";
import { LidarCommandSchema } from "../Modules/lidar";



export const ModuleCommandSchema = z.union([
    z.object({
        Led: LedCommandSchema,
    }),

    // z.object({
    //     StepperMotor: StepperMotorCommandSchema,
    // }),

    // z.object({
    //     Rfid: RfidCommandSchema,
    // }),

    z.object({
        Lidar: LidarCommandSchema,
    }),
]);

export type ModuleCommand = z.infer<
    typeof ModuleCommandSchema
>;

export const IncomingCommandSchema = z.object({
    id: z.string(),
    command: ModuleCommandSchema,
});

export type IncomingCommand = z.infer<
    typeof IncomingCommandSchema
>;