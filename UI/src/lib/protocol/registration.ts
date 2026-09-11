import { z } from "zod";
import { ModuleTypeSchema } from "./module-type";

export const RegistrationSchema = z.object({
    id: z.string(),
    module_type: ModuleTypeSchema,
    lool_up_id: z.string(),
    parent_id: z.string(),
});

export type Registration = z.infer<
    typeof RegistrationSchema
>;