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

import { StepperDial } from "@/components/StepperDial";
import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { useState } from "react";
import { IncomingCommand } from "../IncomingCommand";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { z } from "zod";

export const PivotPointSchema = z.enum(["Min", "Max"]);

export type PivotPoint = z.infer<typeof PivotPointSchema>;

export const StepperStateTypeSchema = z.enum([
  "Idle",
  "Moving",
  "Homing",
  "Pivot",
]);

export type StepperStateType = z.infer<typeof StepperStateTypeSchema>;

export const StepperMotorCommandSchema = z.union([
  z.object({
    SetPivotMin: z.object({
      pivot_min: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({
    SetPivotMax: z.object({
      pivot_max: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({ MoveToOrigin: z.object({}) }),

  z.object({
    MoveToAngle: z.object({
      angle: z.number().min(-3.4028234663852886e38).max(3.4028234663852886e38),
    }),
  }),

  z.object({ MoveToPivotMin: z.object({}) }),

  z.object({ MoveToPivotMax: z.object({}) }),

  z.object({
    SetMode: z.object({
      mode: StepperStateTypeSchema,
    }),
  }),
]);

export type StepperMotorCommand = z.infer<typeof StepperMotorCommandSchema>;

export const StepperMotorEventSchema = z.union([
  z.object({
    GetAngle: z.object({
      angle: z.number().min(-3.4028234663852886e38).max(3.4028234663852886e38),
      step: z.number().min(-3.4028234663852886e38).max(3.4028234663852886e38),
    }),
  }),

  z.object({
    GetPivotMin: z.object({
      pivot_min: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({
    GetPivotMax: z.object({
      pivot_max: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38),
    }),
  }),

  z.object({
    GetMode: z.object({
      mode: StepperStateTypeSchema,
    }),
  }),

  z.object({
    GetOrigin: z.object({
      origin: z
        .number()
        .min(-3.4028234663852886e38)
        .max(3.4028234663852886e38)
        .nullable(),
    }),
  }),

  z.object({
    GetPivotPoint: z.object({
      pivot_point: PivotPointSchema,
    }),
  }),
]);

export type StepperMotorEvent = z.infer<typeof StepperMotorEventSchema>;

type StepperInstance = {
  id: string;
  kind: "StepperMotor";
  look_up_id: string;
  state: {
    GetAngle: { angle: number; step: number } | null;
    GetPivotMin: { pivot_min: number } | null;
    GetPivotMax: { pivot_max: number } | null;
    GetMode: { mode: StepperStateType } | null;
    GetOrigin: { origin: number | null } | null;
    GetPivotPoint: { pivot_point: PivotPoint } | null;
  };
};
export interface StepperModule extends StepperInstance {
  handleEvent: (event: StepperMotorEvent) => void;
  setPivotMin: (pivot_min: number) => Promise<unknown>;
  setPivotMax: (pivot_max: number) => Promise<unknown>;
  moveToOrigin: () => Promise<unknown>;
  moveToAngle: (angle: number) => Promise<unknown>;
  moveToPivotMin: () => Promise<unknown>;
  moveToPivotMax: () => Promise<unknown>;
  setMode: (mode: StepperStateType) => Promise<unknown>;
}
export function createStepper(data: StepperInstance): StoreApi<StepperModule> {
  return createStore<StepperModule>((set, get) => ({
    ...data,
    kind: "StepperMotor",
    setPivotMin: (pivot_min: number) =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({
            SetPivotMin: { pivot_min },
          }),
        },
      }),
    setPivotMax: (pivot_max: number) =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({
            SetPivotMax: { pivot_max },
          }),
        },
      }),
    moveToOrigin: () =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ MoveToOrigin: {} }),
        },
      }),
    moveToAngle: (angle: number) =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({
            MoveToAngle: { angle },
          }),
        },
      }),
    moveToPivotMin: () =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ MoveToPivotMin: {} }),
        },
      }),
    moveToPivotMax: () =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ MoveToPivotMax: {} }),
        },
      }),
    setMode: (mode: StepperStateType) =>
      IncomingCommand({
        id: data.id,
        command: {
          StepperMotor: StepperMotorCommandSchema.parse({ SetMode: { mode } }),
        },
      }),
    handleEvent: (event) => {
      if ("GetAngle" in event)
        set({ state: { ...get().state, GetAngle: event.GetAngle } });
      if ("GetPivotMin" in event)
        set({ state: { ...get().state, GetPivotMin: event.GetPivotMin } });
      if ("GetPivotMax" in event)
        set({ state: { ...get().state, GetPivotMax: event.GetPivotMax } });
      if ("GetMode" in event)
        set({ state: { ...get().state, GetMode: event.GetMode } });
      if ("GetOrigin" in event)
        set({ state: { ...get().state, GetOrigin: event.GetOrigin } });
      if ("GetPivotPoint" in event)
        set({ state: { ...get().state, GetPivotPoint: event.GetPivotPoint } });
    },
  }));
}
export const StepperView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) return <div>Waiting for Stepper {id}...</div>;
  return <RegisteredStepperView moduleId={moduleId} />;
};
export const RegisteredStepperView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "StepperMotor") return null;
  return <StepperControls moduleStore={store as StoreApi<StepperModule>} />;
};
function StepperControls({
  moduleStore,
}: {
  moduleStore: StoreApi<StepperModule>;
}) {
  const id = useStore(moduleStore, (s) => s.id);
  const state = useStore(moduleStore, (s) => s.state);
  const connected = useModuleFront((s) => s.PortStat === "Connected");
  const [error, setError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [sent, setSent] = useState(false);
  const send = async (action: () => Promise<unknown>) => {
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
  const disabled = !connected || sending;

  const moveToAngle = useStore(moduleStore, (s) => s.moveToAngle);
  const moveToPivotMin = useStore(moduleStore, (s) => s.moveToPivotMin);
  const moveToPivotMax = useStore(moduleStore, (s) => s.moveToPivotMax);
  const setPivotMin = useStore(moduleStore, (s) => s.setPivotMin);
  const setPivotMax = useStore(moduleStore, (s) => s.setPivotMax);
  const setMode = useStore(moduleStore, (s) => s.setMode);
  const [target, setTarget] = useState("");
  const [minimum, setMinimum] = useState("");
  const [maximum, setMaximum] = useState("");
  const [mode, setModeDraft] = useState<StepperStateType | null>(null);
  const targetValid =
    target.trim() !== "" &&
    StepperMotorCommandSchema.safeParse({
      MoveToAngle: { angle: Number(target) },
    }).success;
  const reportedAngle = state.GetAngle?.angle ?? null;
  const commit = (angle: number) => {
    if (
      !disabled &&
      StepperMotorCommandSchema.safeParse({ MoveToAngle: { angle } }).success
    )
      void send(() => moveToAngle(angle));
  };
  return (
    <ModuleCard type="Stepper" id={id}>
      <div className="space-y-5">
        <div className="flex items-center justify-between gap-3">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            Rotary position
          </span>
          <Badge
            variant={state.GetMode?.mode === "Moving" ? "secondary" : "outline"}
          >
            {state.GetMode?.mode ?? "Mode unknown"}
          </Badge>
        </div>
        <StepperDial
          reportedAngle={reportedAngle}
          steps={state.GetAngle?.step ?? null}
          target={targetValid ? Number(target) : null}
          disabled={disabled}
          onTargetChange={(angle) =>
            setTarget(angle === null ? "" : String(angle))
          }
          onCommit={commit}
        />
        <dl className="grid grid-cols-2 gap-3 rounded-lg bg-muted/40 p-3">
          <div className="min-w-0">
            <dt className="text-xs text-muted-foreground">
              Reported total angle
            </dt>
            <dd className="mt-1 break-all font-mono text-sm">
              {reportedAngle === null
                ? "—"
                : `${reportedAngle.toLocaleString(undefined, { maximumFractionDigits: 1 })}°`}
            </dd>
          </div>
          <div className="min-w-0">
            <dt className="text-xs text-muted-foreground">
              Derived revolutions
            </dt>
            <dd className="mt-1 break-all font-mono text-sm">
              {reportedAngle === null
                ? "—"
                : (reportedAngle / 360).toLocaleString(undefined, {
                    maximumFractionDigits: 3,
                  })}
            </dd>
          </div>
        </dl>
        <Field>
          <FieldLabel htmlFor={`${id}-target`}>
            Target angle{" "}
            <span className="text-muted-foreground">· total degrees</span>
          </FieldLabel>
          <div className="flex gap-2">
            <Input
              id={`${id}-target`}
              className="min-h-9 min-w-0"
              type="number"
              step="any"
              placeholder="Signed, multiple turns"
              value={target}
              onChange={(e) => setTarget(e.target.value)}
              disabled={disabled}
            />
            <Button
              className="min-h-9"
              disabled={disabled || !targetValid}
              onClick={() => commit(Number(target))}
            >
              Move
            </Button>
          </div>
          <FieldDescription>
            Local target only. The dial center shows the last device report.
          </FieldDescription>
        </Field>
        <Button
          className="min-h-9 w-full"
          variant="outline"
          disabled={disabled}
          onClick={() => {
            setTarget("0");
            commit(0);
          }}
        >
          Move to 0°
        </Button>
        <details className="rounded-lg border p-3">
          <summary className="cursor-pointer text-sm font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring">
            Pivot &amp; motion settings
          </summary>
          <div className="mt-4 space-y-4">
            <Field>
              <FieldLabel htmlFor={`${id}-pivot-minimum`}>
                Pivot minimum · °
              </FieldLabel>
              <p className="text-xs text-muted-foreground">
                Reported:{" "}
                {state.GetPivotMin
                  ? `${state.GetPivotMin.pivot_min}°`
                  : "Not reported"}
              </p>
              <div className="flex gap-2">
                <Input
                  id={`${id}-pivot-minimum`}
                  className="min-h-9 min-w-0"
                  type="number"
                  step="any"
                  placeholder="Draft minimum"
                  value={minimum}
                  onChange={(e) => setMinimum(e.target.value)}
                  disabled={disabled}
                />
                <Button
                  className="min-h-9"
                  variant="secondary"
                  disabled={
                    disabled ||
                    !minimum.trim() ||
                    !StepperMotorCommandSchema.safeParse({
                      SetPivotMin: { pivot_min: Number(minimum) },
                    }).success
                  }
                  onClick={() => void send(() => setPivotMin(Number(minimum)))}
                >
                  Apply
                </Button>
              </div>
            </Field>
            <Field>
              <FieldLabel htmlFor={`${id}-pivot-maximum`}>
                Pivot maximum · °
              </FieldLabel>
              <p className="text-xs text-muted-foreground">
                Reported:{" "}
                {state.GetPivotMax
                  ? `${state.GetPivotMax.pivot_max}°`
                  : "Not reported"}
              </p>
              <div className="flex gap-2">
                <Input
                  id={`${id}-pivot-maximum`}
                  className="min-h-9 min-w-0"
                  type="number"
                  step="any"
                  placeholder="Draft maximum"
                  value={maximum}
                  onChange={(e) => setMaximum(e.target.value)}
                  disabled={disabled}
                />
                <Button
                  className="min-h-9"
                  variant="secondary"
                  disabled={
                    disabled ||
                    !maximum.trim() ||
                    !StepperMotorCommandSchema.safeParse({
                      SetPivotMax: { pivot_max: Number(maximum) },
                    }).success
                  }
                  onClick={() => void send(() => setPivotMax(Number(maximum)))}
                >
                  Apply
                </Button>
              </div>
            </Field>
            <div className="grid grid-cols-2 gap-2">
              <Button
                className="min-h-9"
                variant="outline"
                disabled={disabled}
                onClick={() => void send(moveToPivotMin)}
              >
                Move to min
              </Button>
              <Button
                className="min-h-9"
                variant="outline"
                disabled={disabled}
                onClick={() => void send(moveToPivotMax)}
              >
                Move to max
              </Button>
            </div>
            <Separator />
            <Field>
              <FieldLabel htmlFor={`${id}-motion-mode`}>Motion mode</FieldLabel>
              <Select
                value={mode}
                items={StepperStateTypeSchema.options.map((value) => ({
                  value,
                  label: value,
                }))}
                onValueChange={(value) => setModeDraft(value)}
                disabled={disabled}
              >
                <SelectTrigger
                  id={`${id}-motion-mode`}
                  className="min-h-9 w-full"
                >
                  <SelectValue placeholder="Choose a mode" />
                </SelectTrigger>
                <SelectContent>
                  {StepperStateTypeSchema.options.map((value) => (
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
                  if (mode) void send(() => setMode(mode));
                }}
              >
                Apply mode
              </Button>
              <FieldDescription>
                Homing runs the device's pivot cycle; it does not reset the
                position counter.
              </FieldDescription>
            </Field>
            <dl className="grid grid-cols-2 gap-3 text-xs">
              <div>
                <dt className="text-muted-foreground">Reported origin</dt>
                <dd className="mt-1 break-all font-mono">
                  {state.GetOrigin
                    ? (state.GetOrigin.origin ?? "Unset")
                    : "Not reported"}
                </dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Reported pivot point</dt>
                <dd className="mt-1">
                  {state.GetPivotPoint?.pivot_point ?? "Not reported"}
                </dd>
              </div>
            </dl>
          </div>
        </details>
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
