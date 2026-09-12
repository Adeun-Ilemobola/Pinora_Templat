import { set, z } from "zod";

import { createStore, StoreApi } from "zustand/vanilla";
import { useModuleFront } from "../Modulefront";
import { useStore } from "zustand";
import { useEffect, useRef, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
  FieldLegend,
  FieldSeparator,
  FieldSet,
  FieldTitle,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { PivotSlider } from "@/components/PivotSlider";
import { Separator } from "@/components/ui/separator";
export const RangePointSchema = z.object({
  x: z.number(),
  y: z.number(),
  distant: z.number(),
});

export type RangePoint = z.infer<typeof RangePointSchema>;

export const PointSchema = z.object({
  x: z.number(),
  y: z.number(),
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

  z.literal("StartScan"),
  z.literal("StopScan"),
  z.literal("Test"),

  z.object({
    SetStep: z.object({
      step: z.number(),
    }),
  }),

  z.object({
    ChangeMotorAngle: z.object({
      id: z.string(),
      step: z.number(),
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
  handleEvent: (event: any) => void;
  setRoi: (min: Point, max: Point) => void;
  setStep: (step: number) => void;
  setChangeMotorAngle: (id: string, step: number) => void;
  setMovePos: (p: Point) => void;
  setScanState: (state: ScanState) => void;
  startScan: () => void;
  stopScan: () => void;
  test: () => void;
}

export function createLidar(data: LidarInstance): StoreApi<LidarModule> {
  return createStore<LidarModule>(() => ({
    id: data.id,
    kind: "Lidar",
    look_up_id: data.look_up_id,
    state: data.state,
    handleEvent: (event) => {
      // Handle incoming events for the Lidar module
      console.log("Handling event for Lidar:", event);
    },
    setRoi: (min: Point, max: Point) => {},
    setStep: (step: number) => {},
    setChangeMotorAngle: (id: string, step: number) => {},
    setMovePos: (p: Point) => {},
    setScanState: (state: ScanState) => {},
    startScan: () => {},
    stopScan: () => {},
    test: () => {},
  }));
}

export const LidarView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);
  if (!moduleId) {
    return <div>Waiting for Lidar {id}...</div>;
  }
  return <RegisteredLidarView moduleId={moduleId} />;
};

export const RegisteredLidarView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Lidar") return null;
  return <LidarControls lidar={store as StoreApi<LidarModule>} />;
};

function LidarControls({ lidar }: { lidar: StoreApi<LidarModule> }) {
  const lidarId = useStore(lidar, (state) => state.id);
  const state = useStore(lidar, (state) => state.state);
  const setRoi = useStore(lidar, (state) => state.setRoi);
  const setStep = useStore(lidar, (state) => state.setStep);
  const setChangeMotorAngle = useStore(
    lidar,
    (state) => state.setChangeMotorAngle,
  );
  const setMovePos = useStore(lidar, (state) => state.setMovePos);
  const setScanState = useStore(lidar, (state) => state.setScanState);
  const startScan = useStore(lidar, (state) => state.startScan);
  const stopScan = useStore(lidar, (state) => state.stopScan);
  const test = useStore(lidar, (state) => state.test);
  const [draft, setDraft] = useState(state);

  // Load the Lidar State
  const [localRoi, setLocalRoi] = useState(state.Roi);
  const [RoiSelected, setRoiSelected] = useState(false);

  useEffect(() => setDraft(state), [state]);
  return (
    <main className="flex flex-col gap-2">
      <div className="flex flex-row gap-3 justify-center">
        <RoiConfig
          min={draft.Roi.min}
          max={draft.Roi.max}
          onMinChange={(min) => {}}
          onMaxChange={(max) => {}}
        />
        <Dashboard
          SendRoi={() => {}}
          MovePos={() => {}}
          StartScan={() => {}}
          StopScan={() => {}}
          Test={() => {}}
          scanState={draft.ScanState.state}
          RoiSelected={RoiSelected}
          onRoiSelectedChange={setRoiSelected}
        />
      </div>

      <MotorAngleConfig
        motorAngleX={0}
        motorAngleY={0}
        onMotorAngleXChange={(angle) => {}}
        onMotorAngleYChange={(angle) => {}}
      />

      <main className="flex flex-col gap-2 mt-2 bg-amber-200 h-[calc(100vh-10rem)]">
        <PlayGround/>
      </main>
    </main>
  );
}

interface RoiConfigProps {
  min: Point;
  max: Point;
  onMinChange: (min: Point) => void;
  onMaxChange: (max: Point) => void;
}

function RoiConfig({ min, max, onMinChange, onMaxChange }: RoiConfigProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Roi Config</CardTitle>
      </CardHeader>
      <CardContent className=" w-94">
        <div className="flex flex-col gap-2">
          <FieldSet>
            <FieldLegend>Max</FieldLegend>
            <FieldSet className="flex flex-row gap-2">
              <FieldGroup className=" gap-2">
                <FieldLabel>X</FieldLabel>
                <FieldContent>
                  <Input
                    type="number"
                    value={max.x}
                    onChange={(e) =>
                      onMaxChange({ x: e.target.valueAsNumber, y: min.y })
                    }
                  />
                </FieldContent>
              </FieldGroup>
              <FieldGroup className=" gap-2">
                <FieldLabel>Y</FieldLabel>
                <FieldContent>
                  <Input
                    type="number"
                    value={max.x}
                    onChange={(e) =>
                      onMaxChange({ x: e.target.valueAsNumber, y: max.y })
                    }
                  />
                </FieldContent>
              </FieldGroup>
            </FieldSet>
          </FieldSet>

          <FieldSet>
            <FieldLegend>Min</FieldLegend>
            <FieldSet className="flex flex-row gap-2">
              <FieldGroup className=" gap-2">
                <FieldLabel>X</FieldLabel>
                <FieldContent>
                  <Input
                    type="number"
                    value={min.x}
                    onChange={(e) =>
                      onMinChange({ x: e.target.valueAsNumber, y: min.y })
                    }
                  />
                </FieldContent>
              </FieldGroup>
              <FieldGroup className=" gap-2">
                <FieldLabel>Y</FieldLabel>
                <FieldContent>
                  <Input
                    type="number"
                    value={max.x}
                    onChange={(e) =>
                      onMaxChange({ x: e.target.valueAsNumber, y: max.y })
                    }
                  />
                </FieldContent>
              </FieldGroup>
            </FieldSet>
          </FieldSet>
        </div>
      </CardContent>
    </Card>
  );
}

interface DashboardProps {
  SendRoi: () => void;
  MovePos: () => void;
  StartScan: () => void;
  StopScan: () => void;
  Test: () => void;
  scanState: ScanState;
  RoiSelected: boolean;
  onRoiSelectedChange: (selected: boolean) => void;
}

function Dashboard({
  SendRoi,
  MovePos,
  StartScan,
  StopScan,
  Test,
  scanState,
  RoiSelected,
  onRoiSelectedChange,
}: DashboardProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Dashboard</CardTitle>
      </CardHeader>
      <CardContent className=" w-94">
        <div className="flex flex-col gap-2">
          <Button onClick={SendRoi}>Send Roi</Button>
          <Button onClick={MovePos}>Move Pos</Button>
          <Button onClick={StartScan}>Start Scan</Button>
          <Button onClick={StopScan}>Stop Scan</Button>
          <Button onClick={Test}>Test</Button>
        </div>
      </CardContent>
    </Card>
  );
}

interface MotorAngleConfigProps {
  motorAngleX: number;
  motorAngleY: number;
  onMotorAngleXChange: (angle: number) => void;
  onMotorAngleYChange: (angle: number) => void;
}

function MotorAngleConfig({
  motorAngleX,
  motorAngleY,
  onMotorAngleXChange,
  onMotorAngleYChange,
}: MotorAngleConfigProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Motor Angle Config</CardTitle>
      </CardHeader>
      <CardContent className=" h-33">
        <div className="flex flex-row gap-2 w-full justify-center">
          <div className=" flex flex-col gap-1.5 items-center ">
            <h1 className=" text-2xl">X</h1>
            <Separator />
            <PivotSlider
              value={motorAngleX}
              label="Angle"
              max={90}
              min={-90}
              onValueChange={(v) => {
                onMotorAngleXChange(v);
              }}
              showValue
              step={1}
              className=" w-55 "
            />
          </div>
          <Separator orientation="vertical" />

          <div className=" flex flex-col gap-1.5 items-center ">
            <h1 className=" text-2xl">Y</h1>
            <Separator />
            <PivotSlider
              value={motorAngleY}
              label="Angle"
              max={90}
              min={-90}
              onValueChange={(v) => {
                onMotorAngleYChange(v);
              }}
              showValue
              step={1}
              className=" w-55 "
            />
          </div>
        </div>
      </CardContent>
    </Card>
  );
}

//const CellSize = 5;
const MaxAngle = 180;
const MinAngle = 0;
function Clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function GridToAngle(pos: number, min: number, max: number): number {
  let t = (pos - min) / (max - min);
  const angle = MinAngle + t * (MaxAngle - MinAngle);
   return Clamp(angle, MinAngle, MaxAngle);
}
// function AngleToGrid(A: number) {}
// function pointToIndex(point: Point, width:number): number {
//   return point.x + width * point.y;
// }
function AngleToPivot(point:Point):Point {
  return {
    x: (MaxAngle / 2) - point.x,
    // y: point.y - (MaxAngle / 2),
    y: (MaxAngle / 2) - point.y
  };
}
type mousePosPrpos  ={

  mousePoint:Point,
  anglePoint:Point,
  pivotPoint:Point
}
interface PlayGroundProps {}

function PlayGround({}: PlayGroundProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [hasHovered, SetHasHovered] = useState(false);
  const [mousePos, setMovePos] = useState<mousePosPrpos>({
    mousePoint: { x: 0, y: 0 },
    anglePoint: { x: 0, y: 0 },
    pivotPoint: { x: 0, y: 0 },
  });


  return (
    <div className="relative w-full h-full">
      <canvas
        ref={canvasRef}
        className=" w-full h-full bg-green-600 "
        onPointerEnter={() => {
          SetHasHovered(true);
        }}
        
        onPointerMove={(event) => {
          const rect = event.currentTarget.getBoundingClientRect();

          const x = event.clientX - rect.left;
          const y = event.clientY - rect.top;

          const ax = GridToAngle(x, 0, rect.width);
          const ay = GridToAngle(y, 0, rect.height);

          const newPos = {
            mousePoint: { x, y },
            anglePoint: { x: Math.round(ax), y: Math.round(ay) },
            pivotPoint: AngleToPivot({ x: Math.round(ax), y: Math.round(ay) }),
          };

          setMovePos(newPos);
          console.info(newPos);
        }}
      ></canvas>

      {hasHovered && (
        <div
          className="absolute z-50 pointer-events-none flex flex-col gap-1 rounded-md bg-gray-700/55"
          style={{
            left: `${mousePos.mousePoint.x + 12}px`,
            top: `${mousePos.mousePoint.y + 12}px`,
          }}
        >
          <span>
            (<span>{Math.round(mousePos.mousePoint.x)}</span> ,
            <span>{Math.round(mousePos.mousePoint.y)}</span>)
          </span>

          <span>
            (<span>{mousePos.anglePoint.x}</span> ,
            <span>{mousePos.anglePoint.y}</span>)
          </span>

          <span>
            (<span>{mousePos.pivotPoint.x}</span> ,{" "}
            <span>{mousePos.pivotPoint.y}</span>)
          </span>
        </div>
      )}
    </div>
  );
}
