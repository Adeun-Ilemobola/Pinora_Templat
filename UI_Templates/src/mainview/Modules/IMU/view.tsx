import { memo, useMemo } from "react";
import { Activity, Gauge, Move3d } from "lucide-react";

import ModuleCore from "@/components/ModuleCore";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";

import type { ImuAxes, ImuDefinition } from "./definition";

type ImuCardProps = {
  module: ImuDefinition;
};

type AxisKey = keyof ImuAxes;

const AXES: Array<{
  key: AxisKey;
  label: string;
  colour: string;
  dot: string;
}> = [
  {
    key: "x",
    label: "X",
    colour: "text-rose-400",
    dot: "bg-rose-400",
  },
  {
    key: "y",
    label: "Y",
    colour: "text-emerald-400",
    dot: "bg-emerald-400",
  },
  {
    key: "z",
    label: "Z",
    colour: "text-sky-400",
    dot: "bg-sky-400",
  },
];

function magnitude(axes: ImuAxes) {
  return Math.sqrt(axes.x ** 2 + axes.y ** 2 + axes.z ** 2);
}

function formatValue(value: number, digits = 3) {
  if (!Number.isFinite(value)) return "0.000";
  return value.toFixed(digits);
}

function AxisPanel({
  title,
  subtitle,
  unit,
  axes,
  rawAxes,
  icon,
}: {
  title: string;
  subtitle: string;
  unit: string;
  axes: ImuAxes;
  rawAxes: ImuAxes;
  icon: React.ReactNode;
}) {
  return (
    <section className="min-w-0 rounded-xl border bg-muted/20 p-4">
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <h2 className="flex items-center gap-2 text-sm font-medium">
            {icon}
            {title}
          </h2>
          <p className="mt-1 text-xs text-muted-foreground">{subtitle}</p>
        </div>

        <div className="text-right">
          <p className="font-mono text-lg font-medium tabular-nums">
            {formatValue(magnitude(axes), 2)}
          </p>
          <p className="text-[10px] uppercase tracking-widest text-muted-foreground">
            magnitude
          </p>
        </div>
      </div>

      <div className="grid grid-cols-3 gap-2">
        {AXES.map((axis) => (
          <div
            key={axis.key}
            className="rounded-lg border bg-background/70 px-3 py-2.5"
          >
            <div className="mb-2 flex items-center gap-1.5">
              <span className={`size-1.5 rounded-full ${axis.dot}`} />
              <span className={`text-xs font-semibold ${axis.colour}`}>
                {axis.label}
              </span>
            </div>
            <p className="truncate font-mono text-sm font-medium tabular-nums">
              {formatValue(axes[axis.key])}
            </p>
            <p className="mt-0.5 text-[10px] text-muted-foreground">{unit}</p>
            <p className="mt-2 truncate border-t pt-2 font-mono text-[10px] tabular-nums text-muted-foreground">
              raw {rawAxes[axis.key]}
            </p>
          </div>
        ))}
      </div>
    </section>
  );
}

export const ImuCard = memo(function ImuCard({ module }: ImuCardProps) {
  const orientation = useMemo(() => {
    const { x, y, z } = module.state.accel;
    const roll = Math.atan2(y, z) * (180 / Math.PI);
    const pitch =
      Math.atan2(-x, Math.sqrt(y * y + z * z)) * (180 / Math.PI);

    return {
      pitch: Number.isFinite(pitch) ? pitch : 0,
      roll: Number.isFinite(roll) ? roll : 0,
    };
  }, [module.state.accel]);

  const isLive = module.state.mode === "Idle";

  return (
    <ModuleCore
      id={module.id}
      manuel_id={module.lool_up_id}
      moduletype={module.module_type}
    >
      <div className="w-full min-w-0 sm:w-[36rem]">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <p className="text-sm font-medium">Motion telemetry</p>
            <p className="mt-1 text-xs text-muted-foreground">
              Calibrated MPU acceleration and angular velocity
            </p>
          </div>

          <Badge variant={isLive ? "default" : "outline"} size="lg">
            <span
              className={`mr-1.5 size-1.5 rounded-full ${
                isLive ? "animate-pulse bg-emerald-300" : "bg-muted-foreground"
              }`}
            />
            {module.state.mode}
          </Badge>
        </div>

        <Separator className="my-4" />

        <div className="grid gap-3 sm:grid-cols-[0.85fr_1.15fr]">
          <section className="relative flex min-h-44 flex-col overflow-hidden rounded-xl border bg-gradient-to-br from-primary/15 via-muted/20 to-background p-4">
            <div className="flex items-center gap-2 text-sm font-medium">
              <Move3d className="size-4 text-primary" />
              Orientation
            </div>

            <div className="relative my-4 flex flex-1 items-center justify-center">
              <div className="absolute size-24 rounded-full border border-dashed border-primary/30" />
              <div className="absolute h-px w-28 bg-border" />
              <div className="absolute h-28 w-px bg-border" />
              <div
                className="relative grid h-14 w-20 place-items-center rounded-lg border border-primary/40 bg-primary/15 shadow-lg shadow-primary/10 transition-transform duration-150"
                style={{
                  transform: `rotate(${orientation.roll.toFixed(2)}deg) translateY(${Math.max(-12, Math.min(12, orientation.pitch / 4)).toFixed(2)}px)`,
                }}
              >
                <span className="text-[10px] font-semibold tracking-widest text-primary">
                  IMU
                </span>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-2 text-center">
              <div className="rounded-lg bg-background/60 px-2 py-2">
                <p className="font-mono text-sm tabular-nums">
                  {formatValue(orientation.pitch, 1)}°
                </p>
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Pitch
                </p>
              </div>
              <div className="rounded-lg bg-background/60 px-2 py-2">
                <p className="font-mono text-sm tabular-nums">
                  {formatValue(orientation.roll, 1)}°
                </p>
                <p className="text-[10px] uppercase tracking-wider text-muted-foreground">
                  Roll
                </p>
              </div>
            </div>
          </section>

          <div className="grid gap-3">
            <AxisPanel
              title="Accelerometer"
              subtitle="Linear acceleration"
              unit="g"
              axes={module.state.accel}
              rawAxes={module.state.accel_raw}
              icon={<Activity className="size-4 text-primary" />}
            />
            <AxisPanel
              title="Gyroscope"
              subtitle="Angular velocity"
              unit="°/s"
              axes={module.state.gyro}
              rawAxes={module.state.gyro_raw}
              icon={<Gauge className="size-4 text-primary" />}
            />
          </div>
        </div>
      </div>
    </ModuleCore>
  );
});
