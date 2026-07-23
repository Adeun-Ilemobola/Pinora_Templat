import { ButtonModule as ButtonModuleSchema } from "@/lib/ModuleDefinitionSchema";
import { memo } from "react";
import z from "zod";
import ModuleCore from "./ModuleCore";



export const ButtonModule = memo(function ButtonModule({ module }: { module: z.infer<typeof ButtonModuleSchema> }) {
  const pressed = module.state.on;

  return (
    <ModuleCore
      id={module.id}
      manuel_id={module.lool_up_id}
      moduletype={module.module_type}
    >

      <div className=' flex flex-row items-center gap-2.5'>
        <span
          className={
            'h-4 w-4 rounded-full ' +
            (pressed ? 'bg-green-500 shadow-[0_0_8px_2px] shadow-green-500/60' : 'bg-gray-500')
          }
        />
        <h1 className=' text-xl font-medium'>
          {pressed ? 'Pressed' : 'Released'}
        </h1>
      </div>
    </ModuleCore>
  )
});
