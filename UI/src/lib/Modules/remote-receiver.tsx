import { ParentControlledState } from "@/components/ParentControlledState";
import { Badge } from "@/components/ui/badge";
import { useEffect, useState } from "react";
import { Radio } from "lucide-react";
import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { z } from "zod";

export const RemoteButtonSchema = z.enum([
  "None",
  "Power",
  "VolumeUp",
  "FunctionStop",
  "Previous",
  "PlayPause",
  "Next",
  "Down",
  "VolumeDown",
  "Up",
  "Zero",
  "Equalizer",
  "StopRepeat",
  "One",
  "Two",
  "Three",
  "Four",
  "Five",
  "Six",
  "Seven",
  "Eight",
  "Nine",
]);

export type RemoteButton = z.infer<typeof RemoteButtonSchema>;

export const RemoteButtonEventSchema = z.union([
  z.object({
    Click: z.object({
      key: RemoteButtonSchema,
    }),
  }),
]);

export type RemoteButtonEvent = z.infer<typeof RemoteButtonEventSchema>;

type RemoteReceiverInstance = {
  hasParent: boolean;
  id: string;
  kind: "RemoteReceiver";
  look_up_id: string;
  state: { key: RemoteButton | null; clickCount: number };
};
export interface RemoteReceiverModule extends RemoteReceiverInstance {
  handleEvent: (event: RemoteButtonEvent) => void;
}
export function createRemoteReceiver(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<RemoteReceiverModule> {
  return createStore<RemoteReceiverModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    state: { key: null, clickCount: 0 },
    kind: "RemoteReceiver",
    handleEvent: (event) => {
      set({
        state: {
          ...get().state,
          key: event.Click.key,
          clickCount: get().state.clickCount + 1,
        },
      });
    },
  }));
}
export const RemoteReceiverView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) return <div>Waiting for RemoteReceiver {id}...</div>;
  return <RegisteredRemoteReceiverView moduleId={moduleId} />;
};
export const RegisteredRemoteReceiverView = ({
  moduleId,
}: {
  moduleId: string;
}) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "RemoteReceiver") return null;
  return (
    <RemoteReceiverControls
      moduleStore={store as StoreApi<RemoteReceiverModule>}
    />
  );
};
function RemoteReceiverControls({
  moduleStore,
}: {
  moduleStore: StoreApi<RemoteReceiverModule>;
}) {
  const hasParent = useStore(moduleStore, (s) => s.hasParent);
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
    <ModuleCard type="Remote receiver" id={id}>
      <div className="space-y-5">
        {hasParent && <ParentControlledState />}
        <div className="flex items-center justify-between gap-3">
          <p className="flex items-center gap-2 text-xs uppercase tracking-widest text-muted-foreground">
            <Radio className="size-3.5" />
            Last input
          </p>
          <Badge variant="outline">Read only</Badge>
        </div>
        <div
          className={`flex min-h-24 items-center justify-center rounded-xl border border-b-4 px-4 py-5 text-center motion-safe:transition-colors ${active ? "border-primary/50 bg-primary/10" : "border-border bg-muted/30"}`}
        >
          <span className="break-words font-heading text-2xl font-semibold">
            {state.key
              ? state.key.replace(/([a-z])([A-Z])/g, "$1 $2")
              : "Awaiting input"}
          </span>
        </div>
        <div className="flex items-baseline justify-between">
          <span className="text-sm text-muted-foreground">Received events</span>
          <span className="font-mono text-2xl tabular-nums">
            {state.clickCount}
          </span>
        </div>
        <p className="text-xs text-muted-foreground">
          Latest received key Â· Physical remote input
        </p>
      </div>
    </ModuleCard>
  );
}
