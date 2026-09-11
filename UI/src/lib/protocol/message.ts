import { z } from "zod";

import { RegistrationSchema } from "./registration";
import { EventPackageSchema } from "./event";
import { SystemInfoSchema } from "./system";

export const ProtocolMessageSchema = z.union([
    z.object({
        Registration: RegistrationSchema,
    }),

    z.object({
        ModuleEvent: EventPackageSchema,
    }),

    z.object({
        System: SystemInfoSchema,
    }),
]);

export type ProtocolMessage = z.infer<
    typeof ProtocolMessageSchema
>;
type KeysOfUnion<T> =
    T extends T ? keyof T : never;

export type ProtocolMessageKey =
    KeysOfUnion<ProtocolMessage>

export type ProtocolMessageValue = 
    | z.infer<typeof RegistrationSchema>
    | z.infer<typeof EventPackageSchema>
    | z.infer<typeof SystemInfoSchema>;