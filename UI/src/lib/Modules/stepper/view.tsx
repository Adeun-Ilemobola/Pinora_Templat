import { ParentControlledState } from "@/components/ParentControlledState";
import { Badge } from "@/components/ui/badge";
import { Field, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Separator } from "@/components/ui/separator";
import { Select, SelectTrigger, SelectValue, SelectContent, SelectItem } from "@/components/ui/select";
import { StepperDial } from "@/components/StepperDial";
import { type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { StepperStateTypeSchema, StepperStateType, StepperMotorCommandSchema } from "./schema";
import { StepperModule } from "./store";

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
        {hasParent && <ParentControlledState />}
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
                ? "â€”"
                : `${reportedAngle.toLocaleString(undefined, { maximumFractionDigits: 1 })}Â°`}
            </dd>
          </div>
          <div className="min-w-0">
            <dt className="text-xs text-muted-foreground">
              Derived revolutions
            </dt>
            <dd className="mt-1 break-all font-mono text-sm">
              {reportedAngle === null
                ? "â€”"
                : (reportedAngle / 360).toLocaleString(undefined, {
                    maximumFractionDigits: 3,
                  })}
            </dd>
          </div>
        </dl>
        <Field>
          <FieldLabel htmlFor={`${id}-target`}>
            Target angle{" "}
            <span className="text-muted-foreground">Â· total degrees</span>
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
          Move to 0Â°
        </Button>
        <details className="rounded-lg border p-3">
          <summary className="cursor-pointer text-sm font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring">
            Pivot &amp; motion settings
          </summary>
          <div className="mt-4 space-y-4">
            <Field>
              <FieldLabel htmlFor={`${id}-pivot-minimum`}>
                Pivot minimum Â· Â°
              </FieldLabel>
              <p className="text-xs text-muted-foreground">
                Reported:{" "}
                {state.GetPivotMin
                  ? `${state.GetPivotMin.pivot_min}Â°`
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
                Pivot maximum Â· Â°
              </FieldLabel>
              <p className="text-xs text-muted-foreground">
                Reported:{" "}
                {state.GetPivotMax
                  ? `${state.GetPivotMax.pivot_max}Â°`
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
                <dt className="text-muted-foreground">Last pivot point reached</dt>
                <dd className="mt-1">
                  {state.GetPivotPoint?.pivot_point ?? "Not reported"}
                </dd>
              </div>
            </dl>
          </div>
        </details>
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
