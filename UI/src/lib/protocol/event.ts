import { z } from "zod";
import { LedEventSchema } from "../Modules/Led";
import { LidarEventSchema } from "../Modules/lidar";


// import { ButtonEventSchema } from "../modules/button/protocol";
// import { StepperMotorEventSchema } from "../modules/stepper/protocol";
// import { ImuEventSchema } from "../modules/imu/protocol";
// import { RfidEventSchema } from "../modules/rfid/protocol";
// import { RemoteButtonEventSchema } from "../modules/remote-receiver/protocol";

export const LogPrioritySchema = z.enum([
    "Low",
    "Medium",
    "High",
    "Critical",
]);

export type LogPriority = z.infer<
    typeof LogPrioritySchema
>;

export const SysLogEventSchema = z.strictObject({
    text: z.string(),
    raw_err: z.string().nullable(),
    priority: LogPrioritySchema,
});

export type SysLogEvent = z.infer<
    typeof SysLogEventSchema
>;

export const ModuleEventSchema = z.union([
    z.strictObject({
        Led: LedEventSchema,
    }),

    z.strictObject({
        Lidar: LidarEventSchema,
    }),

    // z.strictObject({
    //     Button: ButtonEventSchema,
    // }),

    z.strictObject({
        SysLog: SysLogEventSchema,
    }),

    // z.strictObject({
    //     StepperMotor: StepperMotorEventSchema,
    // }),

    // z.strictObject({
    //     Imu: ImuEventSchema,
    // }),

    // z.strictObject({
    //     Rfid: RfidEventSchema,
    // }),

    // z.strictObject({
    //     RemoteReceiver: RemoteButtonEventSchema,
    // }),
]);

export type ModuleEvent = z.infer<
    typeof ModuleEventSchema
>;

export const EventPackageSchema = z.strictObject({
    event: ModuleEventSchema,
    id: z.string(),
});

export type EventPackage = z.infer<
    typeof EventPackageSchema
>;