import { RangePoint, Point } from "./schema";

export const CellSize = 5;
export const MinAngle = 0;
export const MaxAngle = 180;
export const GridWidth = MaxAngle - MinAngle + 1;
export const CanvasSize = GridWidth * CellSize;
export const MaxRange = 4000;
export const GapBetweenColors = 100;
export const RangeBucketCount = Math.ceil(MaxRange / GapBetweenColors);
export const Clamp = (value: number, min: number, max: number) =>
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
export const GridCells = GenerateGridCells();
// Undo CSS scaling before selecting a cell; clamp edges to the inclusive angular grid.
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
  if (distance){
     return GetRangeColor(distance, colors.normal);
  }


  if (roi) {
    if (cell.index === hovered) return colors.hover;
    const p = cell.pivotPoint;
    if (
      p.x >= roi.min.x &&
      p.x <= roi.max.x &&
      p.y >= roi.min.y &&
      p.y <= roi.max.y
    )
      return colors.roi;
  }

  return colors.normal;
}
