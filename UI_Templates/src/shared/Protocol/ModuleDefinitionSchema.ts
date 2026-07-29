import z from "zod";
import { ButtonModule } from "@modules/button/definition";
import { LedModule } from "@modules/led/definition";
import { LidarModule } from "@modules/Lidar/definition";
import { RangefinderModule } from "@modules/rangefinder/definition";
import { ServoModule } from "@modules/servo/definition";
import { ModuleEventSchema } from "./ModuleEven";

export const moduleTypeIdentifier = z.enum([
  "Servo",
  "Led",
  "Imu",
  "LedCluster",
  "Button",
  "Lidar",
  "SysLog",
  "Rangefinder",
]);

export type TypeIdentifier = z.infer<typeof moduleTypeIdentifier>;

export type TypeIdentifier_module = Exclude<TypeIdentifier, "SysLog">;
export const RegistrationSchema = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: moduleTypeIdentifier,
});

export const SystemInfoSchema = z.object({
  esp_idf_version: z.string(),
  total_heap: z.string(),
  current_free_heap: z.string(),
  lowest_free_heap: z.string(),
  largest_allocation: z.string(),
  maximum_app_slot: z.string(),
  flash: z.string(),
});
export type SystemInfoType = z.infer<typeof SystemInfoSchema>;


export type Registration = z.infer<typeof RegistrationSchema>;

export const ModuleDefinitionSchema = z.discriminatedUnion("module_type", [
  LedModule,
  ServoModule,
  ButtonModule,
  LidarModule,
  RangefinderModule,
]);

export type ModuleDefinitionType = z.infer<typeof ModuleDefinitionSchema>;

export {
  ButtonModule,
  LedModule,
  LidarModule,
  RangefinderModule,
  ServoModule,
};

export const InComingMessageSchema = z.discriminatedUnion("type", [
    z.object({
        type: z.literal("Registration"),
        payload: RegistrationSchema,
    }),
    z.object({
        type: z.literal("ModuleEvent"),
        payload: ModuleEventSchema,
    }),
    z.object({
        type: z.literal("System"),
        payload: SystemInfoSchema,
    }),
])
export const InComingMessageSchemaType = z.enum(["Registration" ,"ModuleEvent" ,"System"])




type ModuleSelectionState = {
  LookUp_ID_refTo_ID: Record<string, string>;
  modules: Record<string, ModuleDefinitionType>;
};

export function selectModule<T extends TypeIdentifier_module>(
  state: ModuleSelectionState,
  lookupId: string,
  moduleType: T,
) {
  const id = state.LookUp_ID_refTo_ID[lookupId]
  const module = id ? state.modules[id] : undefined
  return module?.module_type === moduleType
    ? module as Extract<typeof module, { module_type: T }>
    : undefined
}
