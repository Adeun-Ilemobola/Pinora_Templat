import { ParentControlledState } from "@/components/ParentControlledState";
import { StoreApi } from "zustand/vanilla";
import { useEffect, useState } from "react";
import { useStore } from "zustand";
import { useModuleFront } from "../../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { Slider } from "@/components/ui/slider";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { LedModule } from "./store";

export const LedView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);

  if (!moduleId) {
    return <div>Waiting for LED {id}...</div>;
  }

  return <RegisteredLedView moduleId={moduleId} />;
};

export const RegisteredLedView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Led") return null;
  return <LedControls led={store as StoreApi<LedModule>} />;
};

function LedControls({ led }: { led: StoreApi<LedModule> }) {
  const hasParent = useStore(led, (state) => state.hasParent);

  const ledId = useStore(led, (state) => state.id);
  const value = useStore(led, (state) => state.state);
  const toggle = useStore(led, (state) => state.toggle);
  const setBrightness = useStore(led, (state) => state.setBrightness);
  const connected = useModuleFront((state) => state.PortStat === "Connected");
  const [draft, setDraft] = useState(value ?? 0);
  useEffect(() => setDraft(value ?? 0), [value]);
  return (
    <ModuleCard type="LED" id={ledId}>
      <div className="space-y-5">
        {hasParent && <ParentControlledState />}
        <div className="flex items-center justify-between">
          <div>
            <p className="text-xs text-muted-foreground">Reported brightness</p>
            <p className="mt-1 text-2xl font-semibold tabular-nums">
              {value ?? "—"}
              {value !== null && (
                <span className="ml-1 text-sm text-muted-foreground">%</span>
              )}
            </p>
          </div>
          <Badge variant={value !== null && value > 0 ? "secondary" : "outline"}>
            {value === null ? "Unknown" : value > 0 ? "On" : "Off"}
          </Badge>
        </div>
        {/* Commit once per gesture; device events remain the source of reported state. */}
        <div className="space-y-3">
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>Set brightness</span>
            <span>{draft}%</span>
          </div>
          <Slider
            aria-label={`Brightness for LED ${ledId}`}
            min={0}
            max={100}
            step={1}
            value={[draft]}
            disabled={hasParent || !connected}
            onValueChange={(v) => setDraft(Array.isArray(v) ? v[0] : v)}
            onValueCommitted={(v) => {
              if (!led.getState().hasParent)
                setBrightness(Array.isArray(v) ? v[0] : v);
            }}
          />
        </div>
        <Button
          variant="outline"
          disabled={hasParent || !connected}
          onClick={() => {
            if (!led.getState().hasParent) toggle();
          }}
        >
          Toggle LED
        </Button>
      </div>
    </ModuleCard>
  );
}
