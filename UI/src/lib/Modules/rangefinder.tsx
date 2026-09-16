import { ParentControlledState } from "@/components/ParentControlledState";
import { Badge } from "@/components/ui/badge";
import { Field, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Separator } from "@/components/ui/separator";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";

import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { useState } from "react";
import { IncomingCommand } from "../IncomingCommand";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { z } from "zod";

export const RangefinderDistanceModeSchema = z.enum(["Short", "Long"]);

export type RangefinderDistanceMode = z.infer<
  typeof RangefinderDistanceModeSchema
>;

export const RangefinderCommandSchema = z.union([
  z.literal("StartRanging"),

  z.literal("StopRanging"),

  z.object({
    SetTimingBudget: z.object({
      milliseconds: z.number().int().min(0).max(65535),
    }),
  }),

  z.object({
    SetDistanceMode: z.object({
      mode: RangefinderDistanceModeSchema,
    }),
  }),
]);

export type RangefinderCommand = z.infer<typeof RangefinderCommandSchema>;

export const RangefinderEventSchema = z.union([
  z.object({
    Range: z.object({
      millimeters: z.number().int().min(0).max(65535),
    }),
  }),

  z.object({
    RangingState: z.object({
      is_ranging: z.boolean(),
    }),
  }),

  z.object({
    TimingBudget: z.object({
      milliseconds: z.number().int().min(0).max(65535),
    }),
  }),

  z.object({
    DistanceMode: z.object({
      mode: RangefinderDistanceModeSchema,
    }),
  }),

  z.object({
    InvalidMeasurement: z.object({
      status: z.string(),
    }),
  }),
]);

export type RangefinderEvent = z.infer<typeof RangefinderEventSchema>;

type RangefinderInstance = {
  hasParent: boolean;
  id: string;
  kind: "Rangefinder";
  look_up_id: string;
  state: {
    Range: { millimeters: number } | null;
    RangingState: { is_ranging: boolean } | null;
    TimingBudget: { milliseconds: number } | null;
    DistanceMode: { mode: RangefinderDistanceMode } | null;
    InvalidMeasurement: { status: string } | null;
  };
};
export interface RangefinderModule extends RangefinderInstance {
  handleEvent: (event: RangefinderEvent) => void;
  startRanging: () => Promise<unknown>;
  stopRanging: () => Promise<unknown>;
  setTimingBudget: (milliseconds: number) => Promise<unknown>;
  setDistanceMode: (mode: RangefinderDistanceMode) => Promise<unknown>;
}
export function createRangefinder(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<RangefinderModule> {
  return createStore<RangefinderModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    kind: "Rangefinder",
    state: {
      Range: null,
      RangingState: null,
      TimingBudget: null,
      DistanceMode: null,
      InvalidMeasurement: null,
    },
    startRanging: () =>
      IncomingCommand({
        id,
        command: {
          Rangefinder: RangefinderCommandSchema.parse("StartRanging"),
        },
      }),
    stopRanging: () =>
      IncomingCommand({
        id,
        command: { Rangefinder: RangefinderCommandSchema.parse("StopRanging") },
      }),
    setTimingBudget: (milliseconds: number) =>
      IncomingCommand({
        id,
        command: {
          Rangefinder: RangefinderCommandSchema.parse({
            SetTimingBudget: { milliseconds },
          }),
        },
      }),
    setDistanceMode: (mode: RangefinderDistanceMode) =>
      IncomingCommand({
        id,
        command: {
          Rangefinder: RangefinderCommandSchema.parse({
            SetDistanceMode: { mode },
          }),
        },
      }),
    handleEvent: (event) => {
      if ("Range" in event)
        set({ state: { ...get().state, Range: event.Range } });
      if ("RangingState" in event)
        set({ state: { ...get().state, RangingState: event.RangingState } });
      if ("TimingBudget" in event)
        set({ state: { ...get().state, TimingBudget: event.TimingBudget } });
      if ("DistanceMode" in event)
        set({ state: { ...get().state, DistanceMode: event.DistanceMode } });
      if ("InvalidMeasurement" in event)
        set({
          state: {
            ...get().state,
            InvalidMeasurement: event.InvalidMeasurement,
          },
        });
    },
  }));
}
export const RangefinderView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) return <div>Waiting for Rangefinder {id}...</div>;
  return <RegisteredRangefinderView moduleId={moduleId} />;
};
export const RegisteredRangefinderView = ({
  moduleId,
}: {
  moduleId: string;
}) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Rangefinder") return null;
  return (
    <RangefinderControls moduleStore={store as StoreApi<RangefinderModule>} />
  );
};
function RangefinderControls({
  moduleStore,
}: {
  moduleStore: StoreApi<RangefinderModule>;
}) {
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

  const start = useStore(moduleStore, (s) => s.startRanging);
  const stop = useStore(moduleStore, (s) => s.stopRanging);
  const setTimingBudget = useStore(moduleStore, (s) => s.setTimingBudget);
  const setDistanceMode = useStore(moduleStore, (s) => s.setDistanceMode);
  const [budget, setBudget] = useState("");
  const [mode, setMode] = useState<RangefinderDistanceMode | null>(null);
  const budgetValid =
    budget.trim() !== "" &&
    RangefinderCommandSchema.safeParse({
      SetTimingBudget: { milliseconds: Number(budget) },
    }).success;
  return (
    <ModuleCard type="Rangefinder" id={id}>
      <div className="space-y-5">
        {hasParent && <ParentControlledState />}
        <div className="flex items-center justify-between gap-3">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            Last valid distance
          </span>
          <Badge
            variant={state.RangingState?.is_ranging ? "secondary" : "outline"}
          >
            {state.RangingState
              ? state.RangingState.is_ranging
                ? "Ranging"
                : "Stopped"
              : "Not reported"}
          </Badge>
        </div>
        <div className="flex flex-wrap items-baseline gap-2">
          <span className="font-heading text-5xl font-semibold tracking-tight tabular-nums">
            {state.Range?.millimeters ?? "—"}
          </span>
          <span className="text-lg text-muted-foreground">mm</span>
        </div>
        <div className="grid grid-cols-2 gap-3 rounded-lg bg-muted/40 p-3">
          <div>
            <p className="text-xs text-muted-foreground">Reported budget</p>
            <p className="mt-1 font-mono text-sm">
              {state.TimingBudget
                ? `${state.TimingBudget.milliseconds} ms`
                : "—"}
            </p>
          </div>
          <div>
            <p className="text-xs text-muted-foreground">Reported mode</p>
            <p className="mt-1 text-sm font-medium">
              {state.DistanceMode?.mode ?? "—"}
            </p>
          </div>
        </div>
        <div className="grid grid-cols-2 gap-2">
          <Button
            className="min-h-9"
            disabled={disabled}
            onClick={() => void send(start)}
          >
            Start ranging
          </Button>
          <Button
            className="min-h-9"
            variant="outline"
            disabled={disabled}
            onClick={() => void send(stop)}
          >
            Stop
          </Button>
        </div>
        <Separator />
        <div className="space-y-4">
          <Field>
            <FieldLabel htmlFor={`${id}-budget`}>
              Timing budget <span className="text-muted-foreground">· ms</span>
            </FieldLabel>
            <div className="flex gap-2">
              <Input
                id={`${id}-budget`}
                className="min-h-9 min-w-0"
                type="number"
                min={0}
                max={65535}
                step={1}
                placeholder="Draft budget"
                value={budget}
                disabled={disabled}
                onChange={(e) => setBudget(e.target.value)}
              />
              <Button
                className="min-h-9"
                variant="secondary"
                disabled={disabled || !budgetValid}
                onClick={() => void send(() => setTimingBudget(Number(budget)))}
              >
                Apply
              </Button>
            </div>
            <FieldDescription>
              Whole milliseconds; the sensor validates supported budgets.
            </FieldDescription>
          </Field>
          <Field>
            <FieldLabel htmlFor={`${id}-distance-mode`}>
              Distance mode
            </FieldLabel>
            <div className="flex gap-2">
              <Select
                value={mode}
                items={RangefinderDistanceModeSchema.options.map((value) => ({
                  value,
                  label: value,
                }))}
                onValueChange={(value) => setMode(value)}
                disabled={disabled}
              >
                <SelectTrigger
                  id={`${id}-distance-mode`}
                  className="min-h-9 min-w-0 flex-1"
                >
                  <SelectValue placeholder="Choose a mode" />
                </SelectTrigger>
                <SelectContent>
                  {RangefinderDistanceModeSchema.options.map((value) => (
                    <SelectItem key={value} value={value}>
                      {value}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <Button
                className="min-h-9"
                variant="secondary"
                disabled={disabled || mode === null}
                onClick={() => {
                  if (mode) void send(() => setDistanceMode(mode));
                }}
              >
                Apply
              </Button>
            </div>
          </Field>
        </div>
        {state.InvalidMeasurement && (
          <div className="rounded-md border p-3">
            <p className="text-xs text-muted-foreground">
              Last invalid measurement
            </p>
            <p className="mt-1 break-words text-sm">
              {state.InvalidMeasurement.status}
            </p>
          </div>
        )}
        {!connected && (
          <p className="text-xs text-muted-foreground">
            Disconnected · Showing last reported values
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
