import z from "zod";
import type { ModuleDefinitionType } from "@shared/Protocol/ModuleDefinitionSchema";

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
    max_chunk: z.number(),
    curr_chunk: z.number(),
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
  mutableStateFields: z.tuple([
    z.literal("map"),
  ]),
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
    mutableStateFields: [
      "map"
    ] as const,
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
      console.debug(
        `
         ---------------------------
        [ event_type : PointMap ]
         size : ${event.map.length}
         curr_chunk :${event.curr_chunk}
         max_chunk :${event.max_chunk}
         ---------------------------
        `
        ,
        event.map
      )

      return {
        ...module,
        state: {
          ...module.state,
          map: [...module.state.map, ...event.map]
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

export const lidarHeatmapColors = [
  "#30123B", // closest / minimum depth
  "#4145AB",
  "#466BE3",
  "#3E8EED",
  "#2FA7D8",
  "#1BB9C4",
  "#18C7A3",
  "#2DD080",
  "#55D45F",
  "#7BD64A",
  "#A2D83D",
  "#C7D83B",
  "#E4D63A",
  "#F5CE38",
  "#FBC234",
  "#FBAE32",
  "#F89430",
  "#F4772E",
  "#EC5B2D",
  "#E34232",
  "#D62F3D",
  "#C5264D",
  "#AA2058",
  "#861D59",
  "#5D174F", // farthest / maximum depth
];

export const roiBorder = "#FFFFFF";
export const GRID_BACKGROUND_COLOUR = "#101014";
export const DEFAULT_CELL_COLOUR = "#29292F";
export const MAX_LIDAR_DISTANCE_MM = 4000;

export function distanceToColour(mm: number): string {
  const clampedMm = Math.max(0, Math.min(MAX_LIDAR_DISTANCE_MM, mm));
  const colourIndex = Math.round(
    (clampedMm / MAX_LIDAR_DISTANCE_MM) * (lidarHeatmapColors.length - 1),
  );

  return lidarHeatmapColors[colourIndex] ?? DEFAULT_CELL_COLOUR;
}
