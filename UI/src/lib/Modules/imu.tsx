import { Badge } from "@/components/ui/badge";
import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { ModuleCard } from "@/components/ModuleCard";
import { z } from "zod";

export const RawAxesSchema = z.object({
  x: z.number().int().min(-32768).max(32767),
  y: z.number().int().min(-32768).max(32767),
  z: z.number().int().min(-32768).max(32767),
});

export type RawAxes = z.infer<typeof RawAxesSchema>;

export const AxesSchema = z.object({
  x: z.number(),
  y: z.number(),
  z: z.number(),
});

export type Axes = z.infer<typeof AxesSchema>;

export const MpuDeviceModeSchema = z.enum(["Collecting", "Idle", "Off"]);

export type MpuDeviceMode = z.infer<typeof MpuDeviceModeSchema>;

export const ImuEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Gyro"),
    raw_axes: RawAxesSchema,
    axes: AxesSchema,
  }),

  z.object({
    event_type: z.literal("Accel"),
    raw_axes: RawAxesSchema,
    axes: AxesSchema,
  }),

  z.object({
    event_type: z.literal("Mode"),
    mode: MpuDeviceModeSchema,
  }),
]);

export type ImuEvent = z.infer<typeof ImuEventSchema>;

type ImuInstance = {
  id: string;
  kind: "Imu";
  look_up_id: string;
  state: {
    Gyro: Extract<ImuEvent, { event_type: "Gyro" }> | null;
    Accel: Extract<ImuEvent, { event_type: "Accel" }> | null;
    Mode: MpuDeviceMode | null;
  };
};
export interface ImuModule extends ImuInstance {
  handleEvent: (event: ImuEvent) => void;
}
export function createImu(data: ImuInstance): StoreApi<ImuModule> {
  return createStore<ImuModule>((set, get) => ({
    ...data,
    kind: "Imu",
    handleEvent: (event) => {
      switch (event.event_type) {
        case "Gyro":
          set({ state: { ...get().state, Gyro: event } });
          break;
        case "Accel":
          set({ state: { ...get().state, Accel: event } });
          break;
        case "Mode":
          set({ state: { ...get().state, Mode: event.mode } });
          break;
      }
    },
  }));
}
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
  const id = useStore(moduleStore, (s) => s.id);
  const state = useStore(moduleStore, (s) => s.state);
  return (
    <ModuleCard type="IMU" id={id}>
      <div className="space-y-5">
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
