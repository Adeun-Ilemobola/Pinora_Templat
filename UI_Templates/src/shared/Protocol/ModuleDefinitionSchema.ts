import z from "zod";
import { ButtonModule } from "../../mainview/Modules/button/definition";
import { LedModule } from "../../mainview/Modules/led/definition";
import { LidarModule } from "../../mainview/Modules/Lidar/definition"; 
import { RangefinderModule } from "../../mainview/Modules/rangefinder/definition"; 
import { ServoModule } from "../../mainview/Modules/servo/definition"; 
import { useModuleStore } from "../../Runtime/ModuleStore"; 
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

type TypeIdentifier_module = Exclude<TypeIdentifier, "SysLog">;
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




export function selectModule<T extends TypeIdentifier_module>(
  state: ReturnType<typeof useModuleStore.getState>,
  lookupId: string,
  moduleType: T,
) {
  const id = state.LookUp_ID_refTo_ID[lookupId]
  const module = id ? state.modules[id] : undefined
  return module?.module_type === moduleType
    ? module as Extract<typeof module, { module_type: T }>
    : undefined
}
