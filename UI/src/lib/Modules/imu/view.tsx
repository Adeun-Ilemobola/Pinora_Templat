import { ParentControlledState } from "@/components/ParentControlledState";
import { Badge } from "@/components/ui/badge";
import { type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { ImuModule } from "./store";

export const ImuView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) return <div>Waiting for Imu {id}...</div>;
  return <RegisteredImuView moduleId={moduleId} />;
};
export const RegisteredImuView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Imu") return null;
  return <ImuControls moduleStore={store as StoreApi<ImuModule>} />;
};
function ImuControls({ moduleStore }: { moduleStore: StoreApi<ImuModule> }) {
  const hasParent = useStore(moduleStore, (s) => s.hasParent);
  const id = useStore(moduleStore, (s) => s.id);
  const state = useStore(moduleStore, (s) => s.state);
  return (
    <ModuleCard type="IMU" id={id}>
      <div className="space-y-5">
        {hasParent && <ParentControlledState />}
        <div className="flex items-center justify-between gap-3">
          <p className="text-xs uppercase tracking-widest text-muted-foreground">
            Motion vectors
          </p>
          <Badge
            variant={state.Mode === "Collecting" ? "secondary" : "outline"}
          >
            {state.Mode ?? "Not reported"}
          </Badge>
        </div>
        {(["Accel", "Gyro"] as const).map((kind) => (
          <section key={kind} className="space-y-2">
            <h3 className="text-sm font-medium">
              {kind === "Accel" ? "Acceleration" : "Gyroscope"}
              <span className="ml-2 text-xs font-normal text-muted-foreground">
                Device-scaled
              </span>
            </h3>
            <dl className="grid grid-cols-3 gap-2">
              {(["x", "y", "z"] as const).map((axis) => (
                <div
                  key={axis}
                  className="min-w-0 rounded-md bg-muted/40 p-2.5"
                >
                  <dt className="text-xs uppercase text-muted-foreground">
                    {axis}
                  </dt>
                  <dd className="mt-1 break-all font-mono text-base tabular-nums">
                    {state[kind]?.axes[axis].toFixed(3) ?? "—"}
                  </dd>
                </div>
              ))}
            </dl>
          </section>
        ))}
        <details className="rounded-md border p-3">
          <summary className="cursor-pointer text-xs font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring">
            Raw sensor counts
          </summary>
          <div className="mt-3 space-y-3">
            {(["Accel", "Gyro"] as const).map((kind) => (
              <div key={kind}>
                <p className="mb-1 text-xs text-muted-foreground">
                  {kind === "Accel" ? "Acceleration" : "Gyroscope"}
                </p>
                <div className="grid grid-cols-3 gap-2">
                  {(["x", "y", "z"] as const).map((axis) => (
                    <span key={axis} className="break-all font-mono text-xs">
                      {axis.toUpperCase()} {state[kind]?.raw_axes[axis] ?? "—"}
                    </span>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </details>
        <p className="text-xs leading-relaxed text-muted-foreground">
          Reported vectors only. Orientation is not supplied by this device.
        </p>
      </div>
    </ModuleCard>
  );
}
