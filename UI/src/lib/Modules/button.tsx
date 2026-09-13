import { Badge } from "@/components/ui/badge";
import { useEffect, useState } from "react";
import { CircleDot } from "lucide-react";
import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { z } from "zod";

export const ButtonEventSchema = z.union([
  z.object({
    Ckick: z.object({}),
  }),
]);

export type ButtonEvent = z.infer<typeof ButtonEventSchema>;

type ButtonInstance = {
  id: string;
  kind: "Button";
  look_up_id: string;
  state: { clickCount: number };
};
export interface ButtonModule extends ButtonInstance {
  handleEvent: (event: ButtonEvent) => void;
}
export function createButton(data: ButtonInstance): StoreApi<ButtonModule> {
  return createStore<ButtonModule>((set, get) => ({
    ...data,
    kind: "Button",
    handleEvent: (event) => {
      if ("Ckick" in event)
        set({
          state: { ...get().state, clickCount: get().state.clickCount + 1 },
        });
    },
  }));
}
export const ButtonView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) return <div>Waiting for Button {id}...</div>;
  return <RegisteredButtonView moduleId={moduleId} />;
};
export const RegisteredButtonView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Button") return null;
  return <ButtonControls moduleStore={store as StoreApi<ButtonModule>} />;
};
function ButtonControls({
  moduleStore,
}: {
  moduleStore: StoreApi<ButtonModule>;
}) {
  const id = useStore(moduleStore, (s) => s.id);
  const state = useStore(moduleStore, (s) => s.state);
  const [active, setActive] = useState(false);
  useEffect(
    () =>
      moduleStore.subscribe((next, previous) => {
        if (next.state !== previous.state) setActive(true);
      }),
    [moduleStore],
  );
  useEffect(() => {
    if (!active) return;
    const timer = setTimeout(() => setActive(false), 400);
    return () => clearTimeout(timer);
  }, [active, state]);
  return (
    <ModuleCard type="Button" id={id}>
      <div className="space-y-5">
        <div className="flex items-center justify-between gap-3">
          <p className="text-xs uppercase tracking-widest text-muted-foreground">
            Physical input
          </p>
          <Badge variant="outline">Read only</Badge>
        </div>
        <div
          className={`flex items-center gap-4 rounded-lg border p-4 motion-safe:transition-colors ${active ? "border-primary/50 bg-primary/10" : "bg-muted/30"}`}
        >
          <div
            aria-hidden="true"
            className={`flex size-14 shrink-0 items-center justify-center rounded-full border-4 border-border ${active ? "bg-primary/15" : "bg-card"}`}
          >
            <CircleDot className="size-6 text-muted-foreground" />
          </div>
          <div>
            <p className="font-heading text-lg font-semibold">
              {active ? "Input received" : "Monitoring input"}
            </p>
            <p className="text-xs text-muted-foreground">
              Pressed / released: unavailable
            </p>
          </div>
        </div>
        <div className="flex items-baseline justify-between gap-3">
          <span className="text-sm text-muted-foreground">Observed events</span>
          <span className="font-mono text-2xl tabular-nums">
            {state.clickCount}
          </span>
        </div>
        <p className="text-xs leading-relaxed text-muted-foreground">
          The device reports input events without a pressed or released value.
        </p>
      </div>
    </ModuleCard>
  );
}
