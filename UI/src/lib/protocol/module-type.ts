import { z } from "zod";

export const ModuleTypeSchema = z.enum([
    "Servo",
    "Led",
    "Imu",
    "LedCluster",
    "Button",
    "Lidar",
    "Rangefinder",
    "SysLog",
    "JoyStick",
    "StepperMotor",
    "Rfid",
    "RemoteReceiver",
]);

export type ModuleType = z.infer<typeof ModuleTypeSchema>;