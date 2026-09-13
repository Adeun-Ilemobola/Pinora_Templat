import { z } from "zod";
import { createStore, type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useEffect, useMemo, useRef, useState } from "react";
import { useModuleFront } from "../Modulefront";
import { IncomingCommand } from "../IncomingCommand";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Field, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { PivotSlider } from "@/components/PivotSlider";
import type { ServoModule } from "./servo";
import type { RangefinderModule } from "./rangefinder";

export const RangePointSchema = z.object({
  x: z.number().int(),
  y: z.number().int(),
  distant: z.number(),
});

export type RangePoint = z.infer<typeof RangePointSchema>;

export const PointSchema = z.object({
  x: z.number().int().min(-90).max(90),
  y: z.number().int().min(-90).max(90),
});

export type Point = z.infer<typeof PointSchema>;

export const ScanStateSchema = z.enum(["Idol", "Scanning", "StopScan"]);

export type ScanState = z.infer<typeof ScanStateSchema>;

export const LidarEventSchema = z.union([
  z.object({
    Roi: z.object({
      min: PointSchema,
      max: PointSchema,
    }),
  }),

  z.object({
    PointMap: z.object({
      max_chunk: z.number(),
      curr_chunk: z.number(),
      map: z.array(RangePointSchema),
    }),
  }),

  z.object({
    Target: z.object({
      point: PointSchema,
    }),
  }),

  z.object({
    ScanState: z.object({
      state: ScanStateSchema,
      scan_time: z.number(),
    }),
  }),
]);

export type LidarEvent = z.infer<typeof LidarEventSchema>;

export const LidarCommandSchema = z.union([
  z.object({
    Roi: z.object({
      min: PointSchema,
      max: PointSchema,
    }),
  }),

  z.object({ StartScan: z.object({}) }),
  z.object({ StopScan: z.object({}) }),
  z.object({ Test: z.object({}) }),

  z.object({
    SetStep: z.object({
      step: z.number().int().min(1).max(180),
    }),
  }),

  z.object({
    ChangeMotorAngle: z.object({
      id: z.string(),
      step: z.number().int().min(-90).max(90),
    }),
  }),

  z.object({
    MovePos: z.object({
      p: PointSchema,
    }),
  }),
]);

export type LidarCommand = z.infer<typeof LidarCommandSchema>;

type LidarInstance = {
  hasParent: boolean;
  id: string;
  kind: string;
  look_up_id: string;
  state: {
    Roi: {
      min: Point;
      max: Point;
    };
    PointMap: {
      max_chunk: number;
      curr_chunk: number;
      map: RangePoint[];
    };
    Target: {
      point: Point;
    };
    ScanState: {
      state: ScanState;
      scan_time: number;
    };
  };
};

export interface LidarModule extends LidarInstance {
  targetReported: boolean;
  handleEvent: (event: LidarEvent) => void;
  setRoi: (min: Point, max: Point) => Promise<unknown>;
  setStep: (step: number) => Promise<unknown>;
  setChangeMotorAngle: (id: string, step: number) => Promise<unknown>;
  setMovePos: (p: Point) => Promise<unknown>;
  startScan: () => Promise<unknown>;
  stopScan: () => Promise<unknown>;
  test: () => Promise<unknown>;
  clear: () => Promise<unknown>;
}

export function createLidar(data: LidarInstance): StoreApi<LidarModule> {
  // Firmware emits ordered, one-based batches, not cumulative maps. A batch
  // number replaces its previous contents; coordinates determine placement.
  const chunks = new Map<number, RangePoint[]>();
  const send = (command: LidarCommand) =>
    IncomingCommand({
      id: data.id,
      command: { Lidar: LidarCommandSchema.parse(command) },
    });
  return createStore<LidarModule>((set, get) => ({
    ...data,
    kind: "Lidar",
    targetReported: false,
    handleEvent: (event) => {
      if ("Roi" in event) set({ state: { ...get().state, Roi: event.Roi } });
      if ("Target" in event)
        set({
          targetReported: true,
          state: { ...get().state, Target: event.Target },
        });
      if ("ScanState" in event) {
        const starting = event.ScanState.state === "Scanning";
        if (starting) chunks.clear();
        set({
          state: {
            ...get().state,
            ScanState: event.ScanState,
            ...(starting
              ? { PointMap: { max_chunk: 0, curr_chunk: 0, map: [] } }
              : {}),
          },
        });
      }
      if ("PointMap" in event) {
        const chunk = event.PointMap;
        if (!Number.isInteger(chunk.curr_chunk) || chunk.curr_chunk < 1) return;
        // Also recover if the scan-start event was missed. The protocol has no
        // scan ID, so a new chunk 1 is the only independent reset boundary.
        if (chunk.curr_chunk === 1) chunks.clear();
        chunks.set(chunk.curr_chunk, chunk.map);
        set({
          state: {
            ...get().state,
            PointMap: {
              ...chunk,
              map: [...chunks.entries()]
                .sort(([a], [b]) => a - b)
                .flatMap(([, points]) => points),
            },
          },
        });
      }
    },
    clear: () => {
      return new Promise((resolve) => {
        chunks.clear();
        set({
          state: {
            ...get().state,
            PointMap: { max_chunk: 0, curr_chunk: 0, map: [] },
          },
        });
        resolve(undefined);
      });
    },
    setRoi: (min, max) => send({ Roi: normalizeRoi(min, max) }),
    setStep: (step) => send({ SetStep: { step } }),
    setChangeMotorAngle: (id, step) => send({ ChangeMotorAngle: { id, step } }),
    setMovePos: (p) => send({ MovePos: { p } }),
    startScan: () => send({ StartScan: {} }),
    stopScan: () => send({ StopScan: {} }),
    test: () => send({ Test: {} }),
  }));
}

export const CellSize = 5;
export const MinAngle = 0;
export const MaxAngle = 180;
export const GridWidth = MaxAngle - MinAngle + 1;
export const CanvasSize = GridWidth * CellSize;
export const MaxRange = 4000;
export const GapBetweenColors = 100;
export const RangeBucketCount = Math.ceil(MaxRange / GapBetweenColors);
const Clamp = (value: number, min: number, max: number) =>
  Math.min(Math.max(value, min), max);
export const pointToIndex = (point: Point, width = GridWidth) =>
  point.x + width * point.y;
// Canvas angle increases right/down. Firmware pivot and raw servo angle are
// different: pivot = 90 - canvas angle; raw servo angle = 90 + pivot.
export const AngleToPivot = (point: Point): Point => ({
  x: 90 - point.x,
  y: 90 - point.y,
});
export const PivotToGrid = AngleToPivot;
export type Roi = { min: Point; max: Point };
export const normalizeRoi = (a: Point, b: Point): Roi => ({
  min: { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y) },
  max: { x: Math.max(a.x, b.x), y: Math.max(a.y, b.y) },
});
export const validRoi = (roi: Roi) =>
  [roi.min.x, roi.min.y, roi.max.x, roi.max.y].every(
    (value) => Number.isInteger(value) && value >= -90 && value <= 90,
  ) &&
  roi.min.x <= roi.max.x &&
  roi.min.y <= roi.max.y;
export const validScanStep = (step: number, roi: Roi) =>
  validRoi(roi) &&
  Number.isInteger(step) &&
  step >= 1 &&
  step <= MaxAngle &&
  (roi.max.x - roi.min.x) % step === 0 &&
  (roi.max.y - roi.min.y) % step === 0;
export type GridCell = { gridPoint: Point; pivotPoint: Point; index: number };
export function GenerateGridCells(): GridCell[] {
  return Array.from({ length: GridWidth * GridWidth }, (_, index) => {
    const gridPoint = {
      x: index % GridWidth,
      y: Math.floor(index / GridWidth),
    };
    return { gridPoint, pivotPoint: AngleToPivot(gridPoint), index };
  });
}
const GridCells = GenerateGridCells();
export function pointerToGrid(
  clientX: number,
  clientY: number,
  rect: { left: number; top: number; width: number; height: number },
): Point {
  return {
    x: Clamp(
      Math.floor(((clientX - rect.left) * CanvasSize) / rect.width / CellSize),
      0,
      GridWidth - 1,
    ),
    y: Clamp(
      Math.floor(((clientY - rect.top) * CanvasSize) / rect.height / CellSize),
      0,
      GridWidth - 1,
    ),
  };
}
export function GetRangeColor(
  distance: number | undefined,
  noData = "transparent",
): string {
  if (distance === undefined || !Number.isFinite(distance) || distance < 0)
    return noData;
  const bucket = Math.min(
    Math.floor(Clamp(distance, 0, MaxRange) / GapBetweenColors),
    RangeBucketCount - 1,
  );
  const fraction = bucket / Math.max(1, RangeBucketCount - 1);
  // One deliberate cyan-to-violet data scale, quantized by GapBetweenColors.
  return `hsl(${185 + fraction * 95} 75% ${62 - fraction * 12}%)`;
}
export function buildRangeLookup(points: RangePoint[]): (number | undefined)[] {
  const ranges = new Array<number | undefined>(GridWidth * GridWidth);
  for (const point of points) {
    const grid = PivotToGrid(point);
    if (
      [grid.x, grid.y].every(
        (v) => Number.isInteger(v) && v >= 0 && v < GridWidth,
      ) &&
      Number.isFinite(point.distant) &&
      point.distant >= 0
    )
      ranges[pointToIndex(grid)] = point.distant;
  }
  return ranges;
}
export function resolveCellColor(
  cell: GridCell,
  distance: number | undefined,
  roi: Roi | null,
  hovered: number | null,
  colors: { normal: string; roi: string; hover: string },
): string {
  if (cell.index === hovered) return colors.hover;
  const p = cell.pivotPoint;
  if (
    roi &&
    p.x >= roi.min.x &&
    p.x <= roi.max.x &&
    p.y >= roi.min.y &&
    p.y <= roi.max.y
  )
    return colors.roi;
  return GetRangeColor(distance, colors.normal);
}

export function LidarView({
  id,
  servoX_id,
  servoY_id,
  range_id,
}: {
  id: string;
  servoX_id: string;
  servoY_id: string;
  range_id: string;
}) {
  const ids = useModuleFront((s) => s.LookUpId);
  const moduleId = ids[id];
  const servoXId = ids[servoX_id];
  const servoYId = ids[servoY_id];
  const rangeId = ids[range_id];
  if (!moduleId || !servoXId || !servoYId || !rangeId) return null;
  return (
    <RegisteredLidarView
      moduleId={moduleId}
      servoX_id={servoXId}
      servoY_id={servoYId}
      range_id={rangeId}
    />
  );
}

export function RegisteredLidarView({
  moduleId,
  servoX_id,
  servoY_id,
  range_id,
}: {
  moduleId: string;
  servoX_id: string;
  servoY_id: string;
  range_id: string;
}) {
  const registry = useModuleFront((s) => s.ModuleRegistry);
  const lidar = registry[moduleId];
  const servoX = registry[servoX_id];
  const servoY = registry[servoY_id];
  const range = registry[range_id];
  if (!lidar || !servoX || !servoY || !range) return null;
  if (
    lidar.getState().kind !== "Lidar" ||
    servoX.getState().kind !== "Servo" ||
    servoY.getState().kind !== "Servo" ||
    range.getState().kind !== "Rangefinder"
  )
    return null;
  return (
    <LidarControls
      key={moduleId}
      lidar={lidar as StoreApi<LidarModule>}
      servoX={servoX as StoreApi<ServoModule>}
      servoY={servoY as StoreApi<ServoModule>}
      range={range as StoreApi<RangefinderModule>}
    />
  );
}

function LidarControls({
  lidar,
  servoX,
  servoY,
  range,
}: {
  lidar: StoreApi<LidarModule>;
  servoX: StoreApi<ServoModule>;
  servoY: StoreApi<ServoModule>;
  range: StoreApi<RangefinderModule>;
}) {
  const module = useStore(lidar);
  const connected = useModuleFront((s) => s.PortStat === "Connected");
  const [localRoi, setLocalRoi] = useState<Roi>(() =>
    normalizeRoi(module.state.Roi.min, module.state.Roi.max),
  );
  const dirty = useRef(false);
  const [roiMode, setRoiMode] = useState(false);
  const [step, setStep] = useState(1);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const inFlight = useRef(false);
  const disabled = !connected || busy;
  const scanning = module.state.ScanState.state === "Scanning";
  useEffect(() => {
    if (!dirty.current)
      setLocalRoi(normalizeRoi(module.state.Roi.min, module.state.Roi.max));
  }, [module.state.Roi]);
  const updateRoi = (roi: Roi) => {
    dirty.current = true;
    setLocalRoi(roi);
  };
  const send = async (action: () => Promise<unknown>) => {
    if (!connected || inFlight.current) return;
    inFlight.current = true;
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (error) {
      setError(String(error));
    } finally {
      inFlight.current = false;
      setBusy(false);
    }
  };

  return (
    <main className="space-y-4">
      <div className="grid gap-4 lg:grid-cols-2">
        <RoiConfig id={module.id} roi={localRoi} onChange={updateRoi} />
        <Card>
          <CardHeader>
            <CardTitle>Dashboard</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between gap-3">
              <Badge variant="secondary">{module.state.ScanState.state}</Badge>
              <span className="text-xs text-muted-foreground">
                {module.state.ScanState.scan_time.toFixed(1)} s ·{" "}
                {module.state.PointMap.map.length.toLocaleString()} samples
              </span>
            </div>
            <Field>
              <div className="flex items-center justify-between gap-3">
                <FieldLabel htmlFor={`${module.id}-roi-mode`}>
                  ROI selection
                </FieldLabel>
                <Switch
                  id={`${module.id}-roi-mode`}
                  checked={roiMode}
                  onCheckedChange={setRoiMode}
                />
              </div>
              <FieldDescription>
                {roiMode
                  ? "Select two corners, then send the ROI."
                  : "Click the canvas to move to that pivot position."}
              </FieldDescription>
            </Field>
            <div className="flex flex-wrap gap-2">
              <Button
                disabled={disabled || scanning || !validRoi(localRoi)}
                onClick={() =>
                  void send(() => module.setRoi(localRoi.min, localRoi.max))
                }
              >
                Send ROI
              </Button>
              <Button
                variant="outline"
                onClick={() => {
                  dirty.current = false;
                  setLocalRoi(
                    normalizeRoi(module.state.Roi.min, module.state.Roi.max),
                  );
                }}
              >
                Use reported ROI
              </Button>
            </div>
            <Field>
              <FieldLabel htmlFor={`${module.id}-step`}>
                Scan step · degrees
              </FieldLabel>
              <Input
                id={`${module.id}-step`}
                type="number"
                min={1}
                max={MaxAngle}
                step={1}
                value={Number.isFinite(step) ? step : ""}
                onChange={(e) => setStep(e.target.valueAsNumber)}
              />
              <FieldDescription>
                Use a whole step that divides both ROI spans. Start sends this
                ROI and step before scanning.
              </FieldDescription>
            </Field>
            <div className="flex flex-wrap gap-2">
              <Button
                disabled={
                  disabled || scanning || !validScanStep(step, localRoi)
                }
                onClick={() =>
                  void send(async () => {
                    await module.clear();
                    await module.setRoi(localRoi.min, localRoi.max);
                    await module.setStep(step);
                    await module.startScan();
                  })
                }
              >
                Start scan
              </Button>
              <Button
                variant="outline"
                disabled={disabled}
                onClick={() => void send(module.stopScan)}
              >
                Stop scan
              </Button>
            </div>
            <RangeTelemetry store={range} />
            {error && (
              <p role="alert" className="text-sm text-destructive">
                {error}
              </p>
            )}
            {!connected && (
              <p className="text-xs text-muted-foreground">
                Disconnected · Last reported readings
              </p>
            )}
          </CardContent>
        </Card>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Motor angles</CardTitle>
        </CardHeader>
        <CardContent className="grid gap-6 sm:grid-cols-2">
          <MotorAngleConfig
            store={servoX}
            label="X pivot"
            disabled={disabled || scanning}
            onChange={(angle) =>
              void send(() =>
                module.setChangeMotorAngle(servoX.getState().id, angle),
              )
            }
          />

          <MotorAngleConfig
            store={servoY}
            label="Y pivot"
            disabled={disabled || scanning}
            onChange={(angle) =>
              void send(() =>
                module.setChangeMotorAngle(servoY.getState().id, angle),
              )
            }
          />
        </CardContent>
      </Card>
      <PlayGround
        points={module.state.PointMap.map}
        roi={validRoi(localRoi) ? localRoi : null}
        roiMode={roiMode}
        onRoiChange={updateRoi}
        target={module.targetReported ? module.state.Target.point : null}
        disabled={disabled || scanning}
        onMove={(p) => void send(() => module.setMovePos(p))}
      />
    </main>
  );
}

function MotorAngleConfig({
  store,
  label,
  disabled,
  onChange,
}: {
  store: StoreApi<ServoModule>;
  label: string;
  disabled: boolean;
  onChange: (angle: number) => void;
}) {
  const angle = useStore(store, (s) => s.state.Angle);
  return (
    <PivotSlider
      value={angle}
      label={label}
      min={-90}
      max={90}
      step={1}
      disabled={disabled}
      onValueChange={onChange}
      showValue
      formatValue={(v) => `${v}°`}
    />
  );
}
function RangeTelemetry({ store }: { store: StoreApi<RangefinderModule> }) {
  const state = useStore(store, (s) => s.state);
  return (
    <p className="text-xs text-muted-foreground">
      Sensor:{" "}
      {state.RangingState
        ? state.RangingState.is_ranging
          ? "Ranging"
          : "Stopped"
        : "Not reported"}{" "}
      · Last separate reading:{" "}
      {state.Range ? `${state.Range.millimeters} mm` : "Not reported"}
    </p>
  );
}
function RoiConfig({
  id,
  roi,
  onChange,
}: {
  id: string;
  roi: Roi;
  onChange: (roi: Roi) => void;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>ROI · pivot degrees</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {(["min", "max"] as const).map((bound) => (
          <div key={bound} className="grid grid-cols-2 gap-3">
            {(["x", "y"] as const).map((axis) => (
              <Field key={axis}>
                <FieldLabel htmlFor={`${id}-${bound}-${axis}`}>
                  {bound === "min" ? "Minimum" : "Maximum"} {axis.toUpperCase()}
                </FieldLabel>
                <Input
                  id={`${id}-${bound}-${axis}`}
                  type="number"
                  min={-90}
                  max={90}
                  step={1}
                  value={
                    Number.isFinite(roi[bound][axis]) ? roi[bound][axis] : ""
                  }
                  onChange={(e) =>
                    onChange({
                      ...roi,
                      [bound]: {
                        ...roi[bound],
                        [axis]: e.target.valueAsNumber,
                      },
                    })
                  }
                />
              </Field>
            ))}
          </div>
        ))}
        <p className="text-xs text-muted-foreground">
          −90° to +90° on both axes. Canvas selection and these fields share one
          draft.
        </p>
        {!validRoi(roi) && (
          <p role="status" className="text-xs text-muted-foreground">
            Enter whole pivot angles with minimum ≤ maximum on each axis.
          </p>
        )}
      </CardContent>
    </Card>
  );
}

export function PlayGround({
  points,
  roi,
  roiMode,
  onRoiChange,
  target,
  disabled,
  onMove,
}: {
  points: RangePoint[];
  roi: Roi | null;
  roiMode: boolean;
  onRoiChange: (roi: Roi) => void;
  target: Point | null;
  disabled: boolean;
  onMove: (point: Point) => void;
}) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [hovered, setHovered] = useState<number | null>(null);
  const [anchor, setAnchor] = useState<Point | null>(null);
  const [keyboardPoint, setKeyboardPoint] = useState<Point>({ x: 90, y: 90 });
  const ranges = useMemo(() => buildRangeLookup(points), [points]);
  const hoveredCell = hovered === null ? null : GridCells[hovered];
  useEffect(() => setAnchor(null), [roiMode, roi]);
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    let frame = 0;
    const draw = () => {
      const ctx = canvas.getContext("2d");
      if (!ctx) return;
      const style = getComputedStyle(canvas);
      const token = (name: string) =>
        style.getPropertyValue(`--${name}`).trim();
      const colors = {
        normal: token("muted"),
        roi: token("primary"),
        hover: token("foreground"),
      };
      const selected =
        roiMode && anchor
          ? normalizeRoi(anchor, hoveredCell?.pivotPoint ?? anchor)
          : roi;
      ctx.fillStyle = token("border");
      ctx.fillRect(0, 0, CanvasSize, CanvasSize);
      for (const cell of GridCells) {
        ctx.fillStyle = resolveCellColor(
          cell,
          ranges[cell.index],
          selected,
          hovered,
          colors,
        );
        ctx.fillRect(
          cell.gridPoint.x * CellSize,
          cell.gridPoint.y * CellSize,
          CellSize - 0.5,
          CellSize - 0.5,
        );
      }
      if (
        target &&
        [target.x, target.y].every(
          (v) => Number.isInteger(v) && v >= -90 && v <= 90,
        )
      ) {
        const p = PivotToGrid(target);
        ctx.strokeStyle = token("foreground");
        ctx.lineWidth = 1.5;
        ctx.strokeRect(
          p.x * CellSize - 2,
          p.y * CellSize - 2,
          CellSize + 4,
          CellSize + 4,
        );
      }
    };
    const schedule = () => {
      cancelAnimationFrame(frame);
      frame = requestAnimationFrame(draw);
    };
    schedule();
    const observer = new MutationObserver(schedule);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class", "style"],
    });
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
    };
  }, [ranges, roi, hovered, hoveredCell, anchor, roiMode, target]);
  const select = (grid: Point) => {
    const pivot = AngleToPivot(grid);
    setKeyboardPoint(grid);
    if (roiMode) {
      if (anchor) {
        onRoiChange(normalizeRoi(anchor, pivot));
        setAnchor(null);
      } else setAnchor(pivot);
    } else if (!disabled) onMove(pivot);
  };
  return (
    <Card>
      <CardHeader>
        <CardTitle>LiDAR Playground</CardTitle>
        <p className="text-xs text-muted-foreground">
          {roiMode
            ? anchor
              ? "Select the second corner · Escape cancels"
              : "Select the first ROI corner"
            : disabled
              ? "Hover to inspect · Movement unavailable"
              : "Click to move · Arrow keys navigate, Enter selects"}
        </p>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="mx-auto w-full" style={{ maxWidth: CanvasSize }}>
          <canvas
            ref={canvasRef}
            width={CanvasSize}
            height={CanvasSize}
            tabIndex={0}
            role="img"
            aria-label="LiDAR angular grid. Arrow keys navigate cells; Enter selects a corner in ROI mode or moves the LiDAR. Escape cancels selection."
            className="block aspect-square h-auto w-full rounded-sm bg-muted outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onPointerMove={(event) => {
              const p = pointerToGrid(
                event.clientX,
                event.clientY,
                event.currentTarget.getBoundingClientRect(),
              );
              setHovered(pointToIndex(p));
            }}
            onPointerLeave={() => setHovered(null)}
            onBlur={() => setHovered(null)}
            onClick={(event) =>
              select(
                pointerToGrid(
                  event.clientX,
                  event.clientY,
                  event.currentTarget.getBoundingClientRect(),
                ),
              )
            }
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                setAnchor(null);
                return;
              }
              if (event.key === "Enter" || event.key === " ") {
                event.preventDefault();
                select(keyboardPoint);
                return;
              }
              const delta: Record<string, Point> = {
                ArrowLeft: { x: -1, y: 0 },
                ArrowRight: { x: 1, y: 0 },
                ArrowUp: { x: 0, y: -1 },
                ArrowDown: { x: 0, y: 1 },
              };
              const d = delta[event.key];
              if (!d) return;
              event.preventDefault();
              const p = {
                x: Clamp(keyboardPoint.x + d.x, 0, GridWidth - 1),
                y: Clamp(keyboardPoint.y + d.y, 0, GridWidth - 1),
              };
              setKeyboardPoint(p);
              setHovered(pointToIndex(p));
            }}
          >
            Your browser needs Canvas support to show the angular map.
          </canvas>
        </div>
        <div
          className="min-h-8 rounded-md bg-muted/40 px-3 py-2 font-mono text-xs text-muted-foreground"
          aria-live="off"
        >
          {hoveredCell ? (
            <>
              Grid / canvas angle ({hoveredCell.gridPoint.x}°,{" "}
              {hoveredCell.gridPoint.y}°) · Pivot ({hoveredCell.pivotPoint.x}°,{" "}
              {hoveredCell.pivotPoint.y}°) · Range{" "}
              {ranges[hoveredCell.index] === undefined
                ? "No data"
                : `${ranges[hoveredCell.index]} mm`}
            </>
          ) : (
            "Hover a cell to inspect its coordinates and range."
          )}
        </div>
        <div
          className="flex h-2 overflow-hidden rounded-full"
          aria-hidden="true"
        >
          {Array.from({ length: RangeBucketCount }, (_, bucket) => (
            <span
              key={bucket}
              className="flex-1"
              style={{
                backgroundColor: GetRangeColor(bucket * GapBetweenColors),
              }}
            />
          ))}
        </div>
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>Near · 0 mm</span>
          <span>{GapBetweenColors} mm bands · ROI overrides range</span>
          <span>Far · {MaxRange} mm</span>
        </div>
      </CardContent>
    </Card>
  );
}
