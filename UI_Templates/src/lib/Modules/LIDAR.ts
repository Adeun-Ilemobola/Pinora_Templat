import z from "zod";
import type { ModuleDefinitionType } from "../ModuleDefinitionSchema";

export const PointSchema = z.object({
  x: z.number().int().min(-90).max(90),
  y: z.number().int().min(-90).max(90),
});

export const RangePointSchema = z.object({
  x: z.number().int(),
  y: z.number().int(),
  distant: z.number().int().nonnegative(),
});

export const LidarCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("Roi"),
    min: PointSchema,
    max: PointSchema,
  }),
  z.object({
    command: z.literal("StartScan"),
  }),
  z.object({
    command: z.literal("StopScan"),
  }),
  z.object({
    command: z.literal("Test"),
  }),
  z.object({
    command: z.literal("SetStep"),
    step: z.number().int().positive(),
  }),
  z.object({
    command: z.literal("ChangeMotorAngle"),
    id: z.string(),
    step: z.number().int(),
  }),
  z.object({
    command: z.literal("MovePos"),
    p: PointSchema,
  }),
]);

export const LidarEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Roi"),
    id: z.string(),
    min: PointSchema,
    max: PointSchema,
  }),
  z.object({
    event_type: z.literal("PointMap"),
    id: z.string(),
    map: z.array(RangePointSchema),
  }),
  z.object({
    event_type: z.literal("Target"),
    id: z.string(),
    point: PointSchema,
  }),
  z.object({
    event_type: z.literal("ScanState"),
    id: z.string(),
    state: z.enum(["Idol", "Scanning", "StopScan"]),
  }),
]);

export const LidarModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: z.literal("Lidar"),
  state: z.object({
    state: z.enum(["Idol", "Scanning", "StopScan"]),
    map: z.array(RangePointSchema),
    ROI: z.object({
      min: PointSchema,
      max: PointSchema,
    }),
  }),
});

export type Point = z.infer<typeof PointSchema>;
export type RangePoint = z.infer<typeof RangePointSchema>;

export function lidarInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): z.infer<typeof LidarModule> {
  return {
    id,
    parent_id,
    module_type: "Lidar",
    lool_up_id,
    state: {
      state: "Idol",
      map: [],
      ROI: {
        min: { x: 0, y: 0 },
        max: { x: 0, y: 0 },
      },
    },
  };
}

export function updateLidar(
  module: ModuleDefinitionType,
  event: z.infer<typeof LidarEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Lidar") return module;

  switch (event.event_type) {
    case "Roi":
      return {
        ...module,
        state: {
          ...module.state,
          ROI: { min: event.min, max: event.max },
        },
      };

    case "PointMap":
      return {
        ...module,
        state: {
          ...module.state,
          map: event.map.map(({ x, y, distant }) => ({ x, y, distant })),
        },
      };

    case "Target":
      return module;

    case "ScanState":
      return {
        ...module,
        state: { ...module.state, state: event.state },
      };
  }
}

const lidarHeatmapColors = [
  "#173F8A", // 0
  "#1769C2", // 1
  "#1598D4", // 2
  "#18C6C8", // 3
  "#27B878", // 4
  "#66C94A", // 5
  "#B5D93D", // 6
  "#F2D63A", // 7
  "#F59A32", // 8
  "#ED5938", // 9
  "#D93658", // 10
];

export const roiBorder = "#000000";
export const GRID_BACKGROUND_COLOUR = "#18181b";
export const DEFAULT_CELL_COLOUR = "#3f3f46";
export const MAX_LIDAR_DISTANCE_MM = 4000;

export function distanceToColour(mm: number): string {
  const clampedMm = Math.max(0, Math.min(MAX_LIDAR_DISTANCE_MM, mm));
  const colourIndex = Math.round(
    (clampedMm / MAX_LIDAR_DISTANCE_MM) * (lidarHeatmapColors.length - 1),
  );

  return lidarHeatmapColors[colourIndex] ?? DEFAULT_CELL_COLOUR;
}
