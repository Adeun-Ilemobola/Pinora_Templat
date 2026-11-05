import { type StoreApi } from "zustand/vanilla";
import { useStore } from "zustand";
import { useEffect, useMemo, useRef, useState } from "react";
import { useModuleFront } from "../../Modulefront";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Field, FieldLabel, FieldDescription } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { PivotSlider } from "@/components/PivotSlider";
import type { ServoModule } from "../servo";
import type { RangefinderModule } from "../rangefinder";
import { RangePoint, Point } from "./schema";
import { LidarModule } from "./store";
import { CellSize, MaxAngle, GridWidth, CanvasSize, MaxRange, GapBetweenColors, RangeBucketCount, Clamp, pointToIndex, AngleToPivot, PivotToGrid, Roi, normalizeRoi, validRoi, validScanStep, GridCells, pointerToGrid, GetRangeColor, buildRangeLookup, resolveCellColor } from "./geometry";

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
  // Reports refresh the draft until a local edit; an explicit reset resumes synchronization.
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
    <main className="space-y-6">
      <header className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">LiDAR</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Scan coverage, inspect range samples, and configure the region of
            interest.
          </p>
        </div>
        <Badge variant="outline">
          {connected ? "Connected" : "Disconnected"} Â· {module.id}
        </Badge>
      </header>
      <div className="grid items-start gap-4 xl:grid-cols-[minmax(0,1.4fr)_minmax(20rem,1fr)]">
        <PlayGround
          points={module.state.PointMap.map}
          roi={validRoi(localRoi) ? localRoi : null}
          roiMode={roiMode}
          onRoiChange={updateRoi}
          target={module.targetReported ? module.state.Target.point : null}
          disabled={disabled || scanning}
          onMove={(p) => void send(() => module.setMovePos(p))}
        />
        <div className="grid gap-4">
          <RoiConfig id={module.id} roi={localRoi} onChange={updateRoi} />
          <Card>
            <CardHeader>
              <CardTitle>Scan controls</CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between gap-3">
                <Badge variant="secondary">
                  {module.state.ScanState.state}
                </Badge>
                <span className="text-xs text-muted-foreground">
                  {module.state.ScanState.scan_time.toFixed(1)} s Â·{" "}
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
                  Scan step Â· degrees
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
                      // Send draft bounds and a compatible step before starting the firmware scan.
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
                  Disconnected Â· Last reported readings
                </p>
              )}
            </CardContent>
          </Card>
        </div>
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
      value={angle ?? 0}
      label={label}
      min={-90}
      max={90}
      step={1}
      disabled={disabled}
      onValueChange={onChange}
      showValue
      formatValue={(v) => `${v}Â°`}
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
      Â· Last separate reading:{" "}
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
        <CardTitle>ROI Â· pivot degrees</CardTitle>
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
          âˆ’90Â° to +90Â° on both axes. Canvas selection and these fields share one
          draft.
        </p>
        {!validRoi(roi) && (
          <p role="status" className="text-xs text-muted-foreground">
            Enter whole pivot angles with minimum â‰¤ maximum on each axis.
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
  // A mode switch or replacement draft invalidates an unfinished two-corner selection.
  useEffect(() => setAnchor(null), [roiMode, roi]);
  // Paint scan data, hover, ROI preview, and the reported target using one
  // coordinate system and theme snapshot.
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
      // Keep the first corner fixed; hover previews the second without committing the draft.
      const previewRoi =
        roiMode && anchor
          ? normalizeRoi(anchor, hoveredCell?.pivotPoint ?? anchor)
          : roi;
      ctx.fillStyle = token("border");
      ctx.fillRect(0, 0, CanvasSize, CanvasSize);
      // Color precedence remains hover, ROI, measured range, then empty cell.
      for (const cell of GridCells) {
        ctx.fillStyle = resolveCellColor(
          cell,
          ranges[cell.index],
          previewRoi,
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
        const targetGrid = PivotToGrid(target);
        ctx.strokeStyle = token("foreground");
        ctx.lineWidth = 1.5;
        ctx.strokeRect(
          targetGrid.x * CellSize - 2,
          targetGrid.y * CellSize - 2,
          CellSize + 4,
          CellSize + 4,
        );
      }
    };
    // Coalesce redraws and repaint on theme changes; clean up both on rerender.
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
  // Pointer and keyboard selection share grid-to-pivot conversion and ROI commit logic.
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
        <CardTitle>Scan map</CardTitle>
        <p className="text-xs text-muted-foreground">
          {roiMode
            ? anchor
              ? "Select the second corner Â· Escape cancels"
              : "Select the first ROI corner"
            : disabled
              ? "Hover to inspect Â· Movement unavailable"
              : "Click to move Â· Arrow keys navigate, Enter selects"}
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
              const pointerGrid = pointerToGrid(
                event.clientX,
                event.clientY,
                event.currentTarget.getBoundingClientRect(),
              );
              setHovered(pointToIndex(pointerGrid));
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
              Grid / canvas angle ({hoveredCell.gridPoint.x}Â°,{" "}
              {hoveredCell.gridPoint.y}Â°) Â· Pivot ({hoveredCell.pivotPoint.x}Â°,{" "}
              {hoveredCell.pivotPoint.y}Â°) Â· Range{" "}
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
          <span>Near Â· 0 mm</span>
          <span>{GapBetweenColors} mm bands Â· ROI overrides range</span>
          <span>Far Â· {MaxRange} mm</span>
        </div>
      </CardContent>
    </Card>
  );
}
