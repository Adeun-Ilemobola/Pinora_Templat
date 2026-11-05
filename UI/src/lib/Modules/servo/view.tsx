import { ParentControlledState } from "@/components/ParentControlledState";
import { StoreApi } from "zustand/vanilla";
import { ModuleCard } from "@/components/ModuleCard";
import { useStore } from "zustand";
import { useModuleFront } from "../../Modulefront";
import { PivotSlider } from "@/components/PivotSlider";
import { ServoModule } from "./store";

export const ServoView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);

  if (!moduleId) {
    return <div>Waiting for Servo {id}...</div>;
  }

  return <RegisteredServoView moduleId={moduleId} />;
};

export const RegisteredServoView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Servo") return null;
  return <ServoControls servo={store as StoreApi<ServoModule>} />;
};

function ServoControls({ servo }: { servo: StoreApi<ServoModule> }) {
  const hasParent = useStore(servo, (s) => s.hasParent);
  const servoId = useStore(servo, (state) => state.id);
  const state = useStore(servo, (state) => state.state);
  const setAngle = useStore(servo, (state) => state.setAngle);
  const connected = useModuleFront((state) => state.PortStat === "Connected");
  return (
    <ModuleCard type="Servo" id={servoId}>
      <div className="space-y-5">
        {hasParent && <ParentControlledState />}
        <div>
          <p className="text-xs uppercase tracking-widest text-muted-foreground">
            Reported angle
          </p>
          <p className="mt-2 font-heading text-4xl font-semibold tabular-nums">
            {state.Angle === null ? "Not reported" : state.Angle.toFixed(1)}
            <span className="ml-1 text-xl text-muted-foreground">Â°</span>
          </p>
        </div>
        <PivotSlider
          value={state.Angle ?? 0}
          label="Set angle"
          min={-90}
          max={90}
          pivot={0}
          step={1}
          onValueChange={(angle) => {
            if (!servo.getState().hasParent) setAngle(angle);
          }}
          disabled={hasParent || !connected}
          formatValue={(value) => `${value}Â°`}
          showValue
        />
        <div className="flex justify-between text-xs tabular-nums text-muted-foreground">
          <span>-90Â°</span>
          <span>0Â° Â· Center</span>
          <span>+90Â°</span>
        </div>
        <p className="text-xs text-muted-foreground">
          {connected
            ? "Angle updates when the device reports."
            : "Disconnected Â· Showing last reported angle"}
        </p>
      </div>
    </ModuleCard>
  );
}
