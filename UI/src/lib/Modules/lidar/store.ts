import { createStore, type StoreApi } from "zustand/vanilla";
import { IncomingCommand } from "../../IncomingCommand";
import { RangePoint, Point, ScanState, LidarEvent, LidarCommandSchema, LidarCommand } from "./schema";
import { normalizeRoi } from "./geometry";

type LidarInstance = {
  hasParent: boolean;
  id: string;
  kind: "Lidar";
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
}

export function createLidar(
  hasParent: boolean,
  id: string,
  look_up_id: string,
): StoreApi<LidarModule> {
  // Firmware emits ordered, one-based batches, not cumulative maps. A batch
  // number replaces its previous contents; coordinates determine placement.
  const chunks = new Map<number, RangePoint[]>();
  const send = (command: LidarCommand) =>
    IncomingCommand({
      id,
      command: { Lidar: LidarCommandSchema.parse(command) },
    });
  return createStore<LidarModule>((set, get) => ({
    hasParent,
    id,
    look_up_id,
    state: {
      Roi: {
        min: { x: 0, y: 0 },
        max: { x: 0, y: 0 },
      },
      PointMap: {
        max_chunk: 0,
        curr_chunk: 0,
        map: [],
      },
      Target: {
        point: { x: 0, y: 0 },
      },
      ScanState: {
        state: "Idol",
        scan_time: 0,
      },
    },
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
        // A replayed "Scanning" (after rediscovery) is the same scan, not a new one.
        const starting =
          event.ScanState.state === "Scanning" &&
          get().state.ScanState.state !== "Scanning";
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

    setRoi: (min, max) => send({ Roi: normalizeRoi(min, max) }),
    setStep: (step) => send({ SetStep: { step } }),
    setChangeMotorAngle: (id, step) => send({ ChangeMotorAngle: { id, step } }),
    setMovePos: (p) => send({ MovePos: { p } }),
    startScan: () => send({ StartScan: {} }),
    stopScan: () => send({ StopScan: {} }),
    test: () => send({ Test: {} }),
  }));
}
