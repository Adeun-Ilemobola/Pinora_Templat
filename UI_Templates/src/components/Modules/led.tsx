import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { Command } from "@/lib/ModuleCommand";
import { LedModule } from "@/lib/Modules/LED";
import { memo, useEffect, useState } from "react";
import z from "zod";
import ModuleCore from "./ModuleCore";

type LedCardProps = {
  module: z.infer<typeof LedModule>;
  sendCommand: (command: Command) => Promise<void>;
};

export const LedCard = memo(function LedCard({
  module,
  sendCommand,
}: LedCardProps) {
  const [data, setData] = useState(module.state.brightness);

  useEffect(() => {
    setData(module.state.brightness);
  }, [module.state.brightness]);

  function sendState() {
    const state = Math.min(Math.max(data, 0), 100);

    void sendCommand({
      id: module.parent_id.length > 5 ? module.parent_id : module.id,
      module_type: "Led",
      payload: {
        command: "SetState",
        state,
      },
    });
  }

  function toggle() {
    void sendCommand({
      id: module.parent_id.length > 5 ? module.parent_id : module.id,
      module_type: "Led",
      payload: {
        command: "Toggle",
      },
    });
  }

  return (
    <ModuleCore
      id={module.id}
      manuel_id={module.lool_up_id}
      moduletype={module.module_type}
    >
      <div className="flex flex-col items-center gap-2.5">
        <h1 className="w-10 text-center text-3xl">{data}</h1>

        <Input
          value={data}
          onChange={(event) => {
            const raw = event.target.value;
            setData(raw === "" ? 0 : parseInt(raw, 10) || 0);
          }}
          onKeyDown={(event) => {
            if (event.key !== "Enter") return;
            event.preventDefault();
            sendState();
          }}
        />

        <Button onClick={toggle}>Toggle</Button>
      </div>
    </ModuleCore>
  );
});
