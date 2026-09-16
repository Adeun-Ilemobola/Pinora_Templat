import { expect, mock, test } from "bun:test";
const transmitted: unknown[] = [];
mock.module("../src/lib/IncomingCommand", () => ({
  IncomingCommand: async (command: unknown) => {
    transmitted.push(command);
  },
}));
const {
  createLidar,
  GenerateGridCells,
  GridWidth,
  CanvasSize,
  CellSize,
  AngleToPivot,
  pointerToGrid,
  normalizeRoi,
  validScanStep,
  GetRangeColor,
  RangeBucketCount,
  GapBetweenColors,
  buildRangeLookup,
  pointToIndex,
  resolveCellColor,
} = await import("../src/lib/Modules/lidar");
const makeStore = () =>
  createLidar(false, "lidar", "lidar");

test("inclusive grid, CSS scaled corners and center, and firmware pivot direction", () => {
  const grid = GenerateGridCells();
  expect(grid.length).toBe(32761);
  expect(GridWidth).toBe(181);
  expect(CanvasSize).toBe(GridWidth * CellSize);
  const rect = { left: 30, top: 50, width: 452.5, height: 452.5 };
  for (const [x, y, expected] of [
    [30, 50, { x: 0, y: 0 }],
    [482.5, 50, { x: 180, y: 0 }],
    [30, 502.5, { x: 0, y: 180 }],
    [482.5, 502.5, { x: 180, y: 180 }],
    [256.25, 276.25, { x: 90, y: 90 }],
  ] as const) {
    expect(pointerToGrid(x, y, rect)).toEqual(expected);
  }
  expect(AngleToPivot({ x: 0, y: 0 })).toEqual({ x: 90, y: 90 });
  expect(AngleToPivot({ x: 90, y: 90 })).toEqual({ x: 0, y: 0 });
  expect(grid.at(-1)?.pivotPoint).toEqual({ x: -90, y: -90 });
});

test("ROI normalization in all four directions and exact scan step termination", () => {
  for (const [a, b] of [
    [
      { x: -30, y: -20 },
      { x: 10, y: 40 },
    ],
    [
      { x: 10, y: 40 },
      { x: -30, y: -20 },
    ],
    [
      { x: -30, y: 40 },
      { x: 10, y: -20 },
    ],
    [
      { x: 10, y: -20 },
      { x: -30, y: 40 },
    ],
  ]) {
    const roi = normalizeRoi(a, b);
    expect(roi).toEqual({ min: { x: -30, y: -20 }, max: { x: 10, y: 40 } });
    expect(validScanStep(2, roi)).toBe(true);
    expect(validScanStep(3, roi)).toBe(false);
    expect(validScanStep(0, roi)).toBe(false);
  }
});

test("quantized range bands include 4000 in last bucket; invalid ranges have no color", () => {
  expect(RangeBucketCount).toBe(40);
  expect(GetRangeColor(0)).toBe(GetRangeColor(99));
  expect(GetRangeColor(100)).not.toBe(GetRangeColor(99));
  expect(GetRangeColor(4000)).toBe(GetRangeColor(3900));
  expect(GetRangeColor(8000)).toBe(GetRangeColor(4000));
  for (const distance of [undefined, NaN, Infinity, -1])
    expect(GetRangeColor(distance, "no-data")).toBe("no-data");
  expect(
    new Set(
      Array.from({ length: RangeBucketCount }, (_, i) =>
        GetRangeColor(i * GapBetweenColors),
      ),
    ).size,
  ).toBe(RangeBucketCount);
});

test("pivot map lookup, invalid coordinates, and hover > ROI > range > default", () => {
  const lookup = buildRangeLookup([
    { x: 90, y: 90, distant: 0 },
    { x: -90, y: -90, distant: 4000 },
    { x: 100, y: 0, distant: 100 },
    { x: 0, y: 0, distant: NaN },
  ]);
  expect(lookup[0]).toBe(0);
  expect(lookup.at(-1)).toBe(4000);
  expect(lookup[pointToIndex({ x: 90, y: 90 })]).toBeUndefined();
  const cell = GenerateGridCells()[0];
  const colors = { normal: "default", roi: "roi", hover: "hover" };
  const roi = normalizeRoi({ x: 90, y: 90 }, { x: 0, y: 0 });
  expect(resolveCellColor(cell, 100, roi, 0, colors)).toBe("hover");
  expect(resolveCellColor(cell, 100, roi, null, colors)).toBe("roi");
  expect(resolveCellColor(cell, 100, null, null, colors)).toBe(
    GetRangeColor(100),
  );
  expect(resolveCellColor(cell, undefined, null, null, colors)).toBe("default");
});

test("chunks accumulate immediately, replace retries, and clear at actual scan boundaries", () => {
  const store = makeStore();
  const emit = store.getState().handleEvent;
  emit({
    PointMap: {
      curr_chunk: 1,
      max_chunk: 99,
      map: [{ x: 90, y: 90, distant: 100 }],
    },
  });
  expect(store.getState().state.PointMap.map.length).toBe(1);
  emit({
    PointMap: {
      curr_chunk: 2,
      max_chunk: 99,
      map: [{ x: 89, y: 90, distant: 200 }],
    },
  });
  emit({
    PointMap: {
      curr_chunk: 2,
      max_chunk: 99,
      map: [{ x: 89, y: 90, distant: 300 }],
    },
  });
  expect(store.getState().state.PointMap.map.map((p) => p.distant)).toEqual([
    100, 300,
  ]);
  emit({ ScanState: { state: "Idol", scan_time: 7 } });
  expect(store.getState().state.PointMap.map.length).toBe(2);
  emit({ ScanState: { state: "Scanning", scan_time: 0 } });
  expect(store.getState().state.PointMap.map).toEqual([]);
  emit({
    PointMap: {
      curr_chunk: 1,
      max_chunk: 0,
      map: [{ x: 0, y: 0, distant: 400 }],
    },
  });
  emit({ ScanState: { state: "StopScan", scan_time: 2 } });
  expect(store.getState().state.PointMap.map.length).toBe(1);
  emit({
    PointMap: {
      curr_chunk: 1,
      max_chunk: 0,
      map: [{ x: 1, y: 1, distant: 500 }],
    },
  });
  expect(store.getState().state.PointMap.map.map((p) => p.distant)).toEqual([
    500,
  ]);
});

test("all commands use actual Rust payloads without optimistic reported state", async () => {
  transmitted.length = 0;
  const store = makeStore();
  const state = store.getState();
  const before = state.state;
  await state.setRoi({ x: 30, y: -10 }, { x: -20, y: 40 });
  await state.setMovePos({ x: 27, y: -18 });
  await state.setStep(2);
  await state.setChangeMotorAngle("child-x", -35);
  await state.startScan();
  await state.stopScan();
  await state.test();
  expect(transmitted).toEqual([
    {
      id: "lidar",
      command: {
        Lidar: { Roi: { min: { x: -20, y: -10 }, max: { x: 30, y: 40 } } },
      },
    },
    { id: "lidar", command: { Lidar: { MovePos: { p: { x: 27, y: -18 } } } } },
    { id: "lidar", command: { Lidar: { SetStep: { step: 2 } } } },
    {
      id: "lidar",
      command: { Lidar: { ChangeMotorAngle: { id: "child-x", step: -35 } } },
    },
    ...["StartScan", "StopScan", "Test"].map((key) => ({
      id: "lidar",
      command: { Lidar: { [key]: {} } },
    })),
  ]);
  expect(store.getState().state).toBe(before);
  expect(() => state.setStep(0)).toThrow();
  expect(() => state.setMovePos({ x: 91, y: 0 })).toThrow();
  state.handleEvent({ Target: { point: { x: 12, y: 13 } } });
  state.handleEvent({
    Roi: { min: { x: -10, y: -20 }, max: { x: 20, y: 30 } },
  });
  expect(store.getState().targetReported).toBe(true);
  expect(store.getState().state.Target.point).toEqual({ x: 12, y: 13 });
  expect(store.getState().state.Roi.max).toEqual({ x: 20, y: 30 });
});
