import z from "zod";
import { ButtonModule } from "./Modules/BUTTON";
import { LedModule } from "./Modules/LED";
import { LidarModule } from "./Modules/LIDAR";
import { RangefinderModule } from "./Modules/RANGEFINDER";
import { ServoModule } from "./Modules/SERVO";
import { useModuleStore } from "./ModuleStore";

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

