import { ParentControlledState } from "@/components/ParentControlledState";
import { Badge } from "@/components/ui/badge";
import { useEffect } from "react";
import { Field, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Separator } from "@/components/ui/separator";
import { ScanLine } from "lucide-react";
import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { useState } from "react";
import { IncomingCommand } from "../IncomingCommand";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { z } from "zod";

export const RfidModeSchema = z.enum(["Read", "Write"]);

export type RfidMode = z.infer<typeof RfidModeSchema>;

export const RfidWriteStateSchema = z.enum(["Good", "Bad"]);

export type RfidWriteState = z.infer<typeof RfidWriteStateSchema>;

export const RfidCommandSchema = z.union([
  z.object({ WriteMode: z.object({}) }),

  z.object({ ReadMode: z.object({}) }),

  z.object({
    WritePayload: z.object({
      data: z.array(z.number().int().min(0).max(255)),
    }),
  }),
]);

export type RfidCommand = z.infer<typeof RfidCommandSchema>;

export const RfidEventSchema = z.union([
  z.object({
    GetCard: z.object({
      card_uid: z.string(),
      card_data: z.string(),
    }),
  }),

  z.object({
    GetMode: z.object({
      mode: RfidModeSchema,
    }),
  }),

  z.object({
    GetWriteState: z.object({
      state: RfidWriteStateSchema,
      info: z.string(),
    }),
  }),
]);

export type RfidEvent = z.infer<typeof RfidEventSchema>;

type RfidInstance = {
  hasParent: boolean;
  id: string;
  kind: "Rfid";
  look_up_id: string;
  state: {
    GetCard: { card_uid: string; card_data: string } | null;
    GetMode: { mode: RfidMode } | null;
    GetWriteState: { state: RfidWriteState; info: string } | null;
  };
};
export interface RfidModule extends RfidInstance {
  handleEvent: (event: RfidEvent) => void;
  readMode: () => Promise<unknown>;
  writeMode: () => Promise<unknown>;
  writePayload: (data: number[]) => Promise<unknown>;
}
export function createRfid(hasParent: boolean, id: string, look_up_id: string): StoreApi<RfidModule> {
  return createStore<RfidModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    state: { GetCard: null, GetMode: null, GetWriteState: null },
    kind: "Rfid",
    readMode: () =>
      IncomingCommand({
        id,
        command: { Rfid: RfidCommandSchema.parse({ ReadMode: {} }) },
      }),
    writeMode: () =>
      IncomingCommand({
        id,
        command: { Rfid: RfidCommandSchema.parse({ WriteMode: {} }) },
      }),
    writePayload: (bytes: number[]) =>
      IncomingCommand({
        id,
        command: {
          Rfid: RfidCommandSchema.parse({ WritePayload: { data: bytes } }),
        },
      }),
    handleEvent: (event) => {
      if ("GetCard" in event)
        set({ state: { ...get().state, GetCard: event.GetCard } });
      if ("GetMode" in event)
        set({ state: { ...get().state, GetMode: event.GetMode } });
      if ("GetWriteState" in event)
        set({ state: { ...get().state, GetWriteState: event.GetWriteState } });
    },
  }));
}
export const RfidView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) return <div>Waiting for Rfid {id}...</div>;
  return <RegisteredRfidView moduleId={moduleId} />;
};
export const RegisteredRfidView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Rfid") return null;
  return <RfidControls moduleStore={store as StoreApi<RfidModule>} />;
};
function RfidControls({ moduleStore }: { moduleStore: StoreApi<RfidModule> }) {
  const hasParent = useStore(moduleStore, (s) => s.hasParent);
  const id = useStore(moduleStore, (s) => s.id);
  const state = useStore(moduleStore, (s) => s.state);
  const connected = useModuleFront((s) => s.PortStat === "Connected");
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const send = async (action: () => Promise<unknown>) => {
    if (moduleStore.getState().hasParent || !connected || sending) return;
    try {
      setError(null);
      setSending(true);
      setSent(false);
      await action();
      setSent(true);
    } catch (error) {
      setError(String(error));
    } finally {
      setSending(false);
    }
  };
  const disabled = hasParent || !connected || sending;

  const readMode = useStore(moduleStore, (s) => s.readMode);
  const writeMode = useStore(moduleStore, (s) => s.writeMode);
  const writePayload = useStore(moduleStore, (s) => s.writePayload);
  const [payload, setPayload] = useState("");
  const bytes = payload
    .trim()
    .split(/[\s,]+/)
    .map(Number);
  const valid =
    /^(?:\d{1,3})(?:[\s,]+\d{1,3})*$/.test(payload.trim()) &&
    RfidCommandSchema.safeParse({ WritePayload: { data: bytes } }).success;
  const [active, setActive] = useState(false);
  useEffect(
    () =>
      moduleStore.subscribe((next, previous) => {
        if (next.state.GetCard !== previous.state.GetCard) setActive(true);
      }),
    [moduleStore],
  );
  useEffect(() => {
    if (!active) return;
    const timer = setTimeout(() => setActive(false), 400);
    return () => clearTimeout(timer);
  }, [active, state.GetCard]);
  return (
    <ModuleCard type="RFID" id={id}>
      <div className="space-y-5">
        {hasParent && <ParentControlledState />}
        <div className="flex items-center justify-between gap-3">
          <p className="text-xs uppercase tracking-widest text-muted-foreground">
            Last scanned tag
          </p>
          <Badge variant="outline">
            {state.GetMode ? `${state.GetMode.mode} mode` : "Mode unknown"}
          </Badge>
        </div>
        <div
          className={`rounded-xl border p-4 motion-safe:transition-colors ${active ? "border-primary/50 bg-primary/10" : "bg-muted/30"}`}
        >
          <ScanLine
            aria-hidden="true"
            className="mb-3 size-5 text-muted-foreground"
          />
          <p className="break-all font-mono text-xl font-medium leading-relaxed">
            {state.GetCard?.card_uid || "No tag received"}
          </p>
          <p className="mt-2 text-xs text-muted-foreground">
            {state.GetCard
              ? "Last received UID Â· Presence is not reported"
              : "Present a tag to the reader"}
          </p>
        </div>
        {state.GetCard && (
          <div>
            <p className="mb-2 text-xs text-muted-foreground">Read payload</p>
            <p className="max-h-32 overflow-auto whitespace-pre-wrap break-all rounded-md bg-muted/40 p-3 font-mono text-xs">
              {state.GetCard.card_data || "Empty payload"}
            </p>
          </div>
        )}
        <div className="flex flex-wrap items-center justify-between gap-2">
          <span className="text-xs text-muted-foreground">
            Last write result
          </span>
          <Badge
            variant={
              state.GetWriteState?.state === "Bad" ? "destructive" : "outline"
            }
          >
            {state.GetWriteState?.state === "Good"
              ? "Succeeded"
              : state.GetWriteState?.state === "Bad"
                ? "Failed"
                : "Not reported"}
          </Badge>
        </div>
        {state.GetWriteState?.info && (
          <p className="break-words text-xs text-muted-foreground">
            {state.GetWriteState.info}
          </p>
        )}
        <Separator />
        <div className="grid grid-cols-2 gap-2">
          <Button
            className="min-h-9"
            variant="outline"
            disabled={disabled}
            onClick={() => void send(readMode)}
          >
            Read mode
          </Button>
          <Button
            className="min-h-9"
            variant="outline"
            disabled={disabled}
            onClick={() => void send(writeMode)}
          >
            Write mode
          </Button>
        </div>
        <Field>
          <FieldLabel htmlFor={`${id}-payload`}>Write payload</FieldLabel>
          <Input
            id={`${id}-payload`}
            className="min-h-9 font-mono"
            placeholder="04 162 60 145"
            value={payload}
            onChange={(e) => setPayload(e.target.value)}
            disabled={disabled}
            aria-describedby={`${id}-payload-help`}
          />
          <FieldDescription id={`${id}-payload-help`}>
            Decimal bytes (0â€“255), separated by spaces or commas. Queued for
            writing when a tag is presented.
          </FieldDescription>
          <Button
            className="min-h-9"
            disabled={disabled || !valid}
            onClick={() => void send(() => writePayload(bytes))}
          >
            Queue write payload
          </Button>
        </Field>
        {!connected && (
          <p className="text-xs text-muted-foreground">
            Disconnected Â· Showing last reported values
          </p>
        )}
        {sent && (
          <p role="status" className="text-xs text-muted-foreground">
            Command sent. Readings update when the device reports.
          </p>
        )}
        {error && (
          <p role="alert" className="break-words text-sm text-destructive">
            {error}
          </p>
        )}
      </div>
    </ModuleCard>
  );
}
